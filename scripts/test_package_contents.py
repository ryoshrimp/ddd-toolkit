"""Check Cargo's actual package file selection without publishing or building."""

from pathlib import Path, PurePosixPath
import subprocess
import unittest


ROOT = Path(__file__).resolve().parent.parent
PACKAGES = (
    ("ddd-toolkit", ROOT, ("src", "docs", "examples")),
    ("ddd-toolkit-core", ROOT / "crates/core", ("src", "tests")),
    ("ddd-toolkit-macro", ROOT / "crates/macro", ("src", "tests")),
)


class PackageContentsTests(unittest.TestCase):
    def test_archives_keep_sources_and_fixtures_without_workspace_tooling(self):
        for name, root, directories in PACKAGES:
            with self.subTest(package=name):
                result = subprocess.run(
                    ["cargo", "package", "--list", "--locked", "--allow-dirty", "-p", name],
                    cwd=ROOT, capture_output=True, text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                files = set(result.stdout.splitlines())
                required = {
                    "Cargo.toml", "Cargo.lock", "CHANGELOG.md",
                    "LICENSE-APACHE", "LICENSE-MIT", "README.md", "README.ja.md",
                }
                for directory in directories:
                    required.update(
                        path.relative_to(root).as_posix()
                        for path in (root / directory).rglob("*") if path.is_file()
                    )
                self.assertFalse(required - files, f"Missing: {sorted(required - files)}")
                forbidden = {".github", "scripts", "crates", "facade-tests", "target"}
                unexpected = sorted(
                    path for path in files
                    if PurePosixPath(path).parts[0] in forbidden
                )
                self.assertFalse(unexpected, f"Unexpected files: {unexpected}")


if __name__ == "__main__":
    unittest.main()
