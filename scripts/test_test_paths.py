"""Regression checks for functional/UI routing without compiling Rust fixtures."""

import itertools
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent


class TestPaths(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.log = self.root / "cargo.jsonl"
        self.env = dict(os.environ, PATH=f"{self.root}:{os.environ['PATH']}")
        self.executable(
            "cargo",
            f"import json, sys\nwith open({str(self.log)!r}, 'a') as log:\n"
            "    log.write(json.dumps(sys.argv[1:]) + '\\n')\n",
        )
        self.executable(
            "rustc",
            "import sys\n"
            f"print({str(self.root)!r} if '--print' in sys.argv else 'mock rustc')\n",
        )

    def executable(self, name, body):
        path = self.root / name
        path.write_text(f"#!{sys.executable}\n{body}")
        path.chmod(0o755)

    def run_script(self, name):
        return subprocess.run(
            ["bash", str(ROOT / "scripts" / name)],
            cwd=self.root,
            env=self.env,
            capture_output=True,
            text=True,
        )

    def cargo_commands(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_functional_matrix_does_not_require_rust_src(self):
        self.executable("rustc", "raise SystemExit('rustc preflight must not run')\n")
        result = self.run_script("check-feature-matrix.sh")
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = self.cargo_commands()
        self.assertEqual(len(commands), 17)  # seven facade pairs + three core runs
        self.assertTrue(all("--locked" in command for command in commands))
        self.assertTrue(all("--ignored" not in command for command in commands))
        self.assertEqual(commands[-3:], [
            ["test", "-p", "ddd-toolkit-core", "--no-default-features", "--locked"],
            ["test", "-p", "ddd-toolkit-core", "--features", "serde", "--locked"],
            ["test", "-p", "ddd-toolkit-core", "--features", "serde_with", "--locked"],
        ])

    def test_ui_requires_rust_src_before_tests(self):
        result = self.run_script("check-ui-tests.sh")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("diagnostic snapshots require rust-src", result.stderr)
        self.assertFalse(any("test" in c for c in self.cargo_commands()))

    def test_ui_runs_all_harnesses_in_isolated_feature_variants(self):
        source = self.root / "lib/rustlib/src/rust/library/alloc/src/vec/mod.rs"
        source.parent.mkdir(parents=True)
        source.touch()
        result = self.run_script("check-ui-tests.sh")
        self.assertEqual(result.returncode, 0, result.stderr)
        commands = [c for c in self.cargo_commands() if c[0] == "test"]
        self.assertEqual(commands, [
            ["test", "-p", "ddd-toolkit-core", "--no-default-features",
             "--tests", "--locked", "--", "--ignored"],
            ["test", "-p", "ddd-toolkit-core", "--features", "serde",
             "--tests", "--locked", "--", "--ignored"],
            ["test", "-p", "ddd-toolkit-core", "--features", "serde_with",
             "--tests", "--locked", "--", "--ignored"],
            ["test", "-p", "ddd-toolkit-macro", "--tests", "--locked", "--", "--ignored"],
        ])

    def test_all_trybuild_harnesses_are_routed_to_ui(self):
        harnesses = []
        for crate in ("core", "macro"):
            for path in (ROOT / "crates" / crate / "tests").glob("*.rs"):
                text = path.read_text()
                if "trybuild::TestCases" not in text:
                    continue
                harnesses.append(path)
                tests = re.findall(r"#\[test\](.*?)fn\s+\w+", text, re.S)
                self.assertTrue(tests, str(path))
                for attributes in tests:
                    self.assertIn(
                        '#[ignore = "UI suite: run scripts/check-ui-tests.sh"]',
                        attributes, str(path),
                    )
        self.assertEqual(len(harnesses), 7)

    def test_required_ci_gate_rejects_failed_or_skipped_dependencies(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        gate = workflow.split("\n  test:\n", 1)[1].split("\n  feature-matrix:", 1)[0]
        self.assertIn("if: ${{ always() }}", gate)
        self.assertIn("needs: [functional-tests, ui-diagnostics]", gate)
        command = re.search(r"run: (.+)", gate).group(1)
        for functional, ui in itertools.product(
            ("success", "failure", "cancelled", "skipped"), repeat=2
        ):
            result = subprocess.run(
                ["bash", "-c", command],
                env=dict(self.env, FUNCTIONAL_RESULT=functional, UI_RESULT=ui),
            )
            self.assertEqual(result.returncode == 0, functional == ui == "success")
        self.assertIn("run: ./scripts/check-ui-tests.sh", workflow)
        release = (ROOT / ".github/workflows/release.yml").read_text()
        verify = release.split("\n  publish:", 1)[0]
        for command in ("cargo test --workspace --all-features --locked",
                        "./scripts/check-feature-matrix.sh", "./scripts/check-ui-tests.sh"):
            self.assertIn(f"run: {command}", verify)
        self.assertIn("needs: verify", release)


if __name__ == "__main__":
    unittest.main()
