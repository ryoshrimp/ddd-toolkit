"""Exercise derive resolution in separate Cargo workspaces (no feature unification)."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SOURCE = '''
#![allow(dead_code, clippy::redundant_field_names)]
use api::AggregateRoot as _;

#[derive(macros::ValueObject)]
#[value_object(serde)]
struct Id(u64);

#[derive(macros::ValueObject)]
#[value_object(serde)]
enum State { Ready { count: u64 } }

#[derive(Debug, Clone, PartialEq, macros::DomainEvent)]
enum Event { Created }

#[derive(Debug, macros::Entity, macros::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Event>,
}

#[test]
fn derives_and_serde_work() {
    fn value_object<T: api::ValueObject>() {}
    fn entity<T: api::Entity>() {}
    fn event<T: api::DomainEvent>() {}
    fn aggregate<T: api::AggregateRoot>() {}
    value_object::<Id>();
    entity::<Order>();
    event::<Event>();
    aggregate::<Order>();
    let id = Id::new(7);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "7");
    assert_eq!(serde_json::from_str::<Id>(&json).unwrap(), id);
    let state = State::new_ready(7);
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(serde_json::from_str::<State>(&json).unwrap(), state);
    let mut order = Order { id: 7, events: vec![] };
    order.record_event(Event::Created);
    assert_eq!(order.pull_events(), vec![Event::Created]);
}
'''


def dependency(alias, package, path, features=()):
    return (f'{alias} = {{ package = "{package}", path = "{path.as_posix()}", '
            f'features = {list(features)!r} }}\n'.replace("'", '"'))


class MacroConsumerTests(unittest.TestCase):
    def check_consumer(self, dependencies, source, package="consumer", error=None, integration=None):
        with tempfile.TemporaryDirectory(prefix="ddd-consumer-") as tmp:
            root = Path(tmp)
            (root / "src").mkdir()
            (root / "Cargo.toml").write_text(
                f'[package]\nname = "{package}"\nversion = "0.0.0"\nedition = "2024"\n'
                '[workspace]\n[dependencies]\n' + dependencies,
                encoding="utf-8",
            )
            (root / "src/lib.rs").write_text(source, encoding="utf-8")
            if integration is not None:
                (root / "tests").mkdir()
                (root / "tests/consumer.rs").write_text(integration, encoding="utf-8")
            env = os.environ.copy()
            env["CARGO_TARGET_DIR"] = str(ROOT / "target/macro-consumers")
            result = subprocess.run(
                ["cargo", "test", "--offline", "--manifest-path", str(root / "Cargo.toml")],
                env=env, capture_output=True, text=True, timeout=180,
            )
            if error:
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(error, result.stderr)
            else:
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_dependency_resolution(self):
        for kind, alias in [
            ("facade", "ddd_toolkit"), ("facade", "renamed_facade"),
            ("core", "ddd_toolkit_core"), ("core", "renamed_core"),
        ]:
            with self.subTest(kind=kind, alias=alias):
                if kind == "facade":
                    deps = dependency(alias, "ddd-toolkit", ROOT, ["serde"])
                    macros = alias
                else:
                    deps = dependency(alias, "ddd-toolkit-core", ROOT / "crates/core", ["serde"])
                    deps += dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
                    # Direct-core users enable serde's derive support explicitly.
                    deps += 'serde = { version = "1", features = ["derive"] }\n'
                    macros = "macros"
                deps += 'serde_json = "1"\n'
                self.check_consumer(deps, SOURCE.replace("api::", f"{alias}::").replace("macros::", f"{macros}::"))

    def test_core_takes_priority(self):
        # Both dependencies exist; core must be preferred even for facade derives.
        deps = dependency("facade", "ddd-toolkit", ROOT)
        deps += dependency("core_api", "ddd-toolkit-core", ROOT / "crates/core", ["serde"])
        deps += 'serde_json = "1"\nserde = { version = "1", features = ["derive"] }\n'
        self.check_consumer(deps, SOURCE.replace("api::", "core_api::").replace("macros::", "facade::"))

    def test_missing_dependency_diagnostic(self):
        deps = dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
        self.check_consumer(
            deps, '#[derive(macros::ValueObject)]\nstruct Id(u64);',
            error="DDD derives require a ddd-toolkit-core or ddd-toolkit dependency; add one to [dependencies] in Cargo.toml",
        )

    def test_self_references(self):
        for package, path in [("ddd-toolkit", ROOT), ("ddd-toolkit-core", ROOT / "crates/core")]:
            with self.subTest(package=package):
                name = package.replace("-", "_")
                deps = dependency("implementation", package, path, ["serde"])
                deps += dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
                deps += 'serde_json = "1"\nserde = { version = "1", features = ["derive"] }\n'
                source = f'extern crate self as {name};\npub use implementation::*;\n'
                source += SOURCE.replace("#![allow(dead_code, clippy::redundant_field_names)]", "").replace("api::", f"{name}::")
                # Test the same derive in library, integration, and doctest targets.
                source += '\n/// ```\n/// #[derive(macros::ValueObject)]\n/// struct Id(u64);\n/// ```\npub struct DocExample;\n'
                self.check_consumer(deps, source, package=package,
                                    integration=SOURCE.replace("api::", f"{name}::"))


if __name__ == "__main__":
    unittest.main()
