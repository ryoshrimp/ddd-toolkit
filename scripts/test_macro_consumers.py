"""Exercise derive resolution in separate Cargo workspaces (no feature unification)."""

import os
import re
import shutil
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SOURCE = '''
#![allow(dead_code)]
use api::AggregateRoot as _;

#[derive(macros::ValueObject)]
#[value_object(serde)]
struct Id(u64);

#[derive(macros::ValueObject)]
#[value_object(serde)]
enum State { Ready { count: u64 }, Code(u64), Codes(Vec<u64>) }

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
    for (state, expected) in [
        (State::new_ready(7), r#"{"Ready":{"count":7}}"#),
        (State::new_code(7), r#"{"Code":7}"#),
        (State::new_codes(vec![7]), r#"{"Codes":[7]}"#),
    ] {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, expected);
        assert_eq!(serde_json::from_str::<State>(&json).unwrap(), state);
    }
    let mut order = Order { id: 7, events: vec![] };
    order.record_event(Event::Created);
    assert_eq!(order.pull_events(), vec![Event::Created]);
}
'''


PLAIN_SOURCE = '''
#![allow(dead_code)]
#[derive(macros::ValueObject)]
struct Id { value: u64 }
#[derive(macros::ValueObject)]
enum State { Ready { count: u64 }, Code(u64) }
#[derive(Debug, Clone, PartialEq, macros::DomainEvent)]
struct Event;
#[derive(Debug, macros::Entity, macros::AggregateRoot)]
struct Order {
    #[entity(id)]
    id: u64,
    #[aggregate(events)]
    events: Vec<Event>,
}
#[test]
fn contracts_work_without_serde() {
    fn value_object<T: api::ValueObject>() {}
    fn aggregate<T: api::AggregateRoot>() {}
    value_object::<Id>();
    value_object::<State>();
    aggregate::<Order>();
    let mut order = Order { id: 1, events: vec![] };
    api::AggregateRoot::record_event(&mut order, Event);
    assert_eq!(api::AggregateRoot::pull_events(&mut order), vec![Event]);
    assert_eq!(Id::new(7).into_inner(), 7);
    assert_eq!(State::new_ready(7), State::Ready { count: 7 });
}
'''
MINIMUM_CORE = "0.4.1"


def dependency(alias, package, path, features=()):
    return (f'{alias} = {{ package = "{package}", path = "{path.as_posix()}", '
            f'features = {list(features)!r} }}\n'.replace("'", '"'))


class MacroConsumerTests(unittest.TestCase):
    def check_consumer(self, dependencies, source, package="consumer", error=None, integration=None, offline=True):
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
            command = ["cargo", "test", "--manifest-path", str(root / "Cargo.toml")]
            if offline:
                command.append("--offline")
            result = subprocess.run(
                command,
                env=env, capture_output=True, text=True, timeout=180,
            )
            if error:
                self.assertNotEqual(result.returncode, 0)
                for message in ([error] if isinstance(error, str) else error):
                    self.assertIn(message, result.stderr)
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

    def test_minimum_core_compatibility(self):
        # Test the released lower bound, never a copy of the current core.
        # Stage the actual facade source/manifest, changing only core selection
        # and paths so facade re-exports also exercise that lower bound.
        with tempfile.TemporaryDirectory(prefix="ddd-minimum-facade-") as tmp:
            facade = Path(tmp)
            shutil.copytree(ROOT / "src", facade / "src")
            manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
            manifest = manifest.replace(
                'members = ["crates/core", "crates/macro", "facade-tests"]', 'members = []',
            )
            manifest = re.sub(
                r'ddd-toolkit-core = \{ version = "[^"]+", path = "crates/core" \}',
                f'ddd-toolkit-core = {{ version = "={MINIMUM_CORE}" }}', manifest,
            )
            manifest = manifest.replace('path = "crates/macro"',
                                        f'path = "{(ROOT / "crates/macro").as_posix()}"')
            self.assertIn(f'version = "={MINIMUM_CORE}"', manifest)
            (facade / "Cargo.toml").write_text(manifest, encoding="utf-8")
            for kind, alias in [
                ("core", "ddd_toolkit_core"), ("core", "renamed_core"),
                ("facade", "ddd_toolkit"), ("facade", "renamed_facade"),
            ]:
                for serde_enabled in (False, True):
                    with self.subTest(kind=kind, alias=alias, serde=serde_enabled):
                        features = ["serde"] if serde_enabled else []
                        if kind == "core":
                            deps = (f'{alias} = {{ package = "ddd-toolkit-core", '
                                    f'version = "={MINIMUM_CORE}", features = {features!r} }}\n').replace("'", '"')
                            deps += dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
                            macros = "macros"
                            if serde_enabled:
                                deps += 'serde = { version = "1", features = ["derive"] }\n'
                        else:
                            deps = dependency(alias, "ddd-toolkit", facade, features)
                            macros = alias
                        if serde_enabled:
                            deps += 'serde_json = "1"\n'
                        source = SOURCE if serde_enabled else PLAIN_SOURCE
                        self.check_consumer(
                            deps, source.replace("api::", f"{alias}::").replace("macros::", f"{macros}::"),
                            offline=False,
                        )

    def test_current_core_without_serde(self):
        for kind, alias in [
            ("core", "ddd_toolkit_core"), ("core", "renamed_core"),
            ("facade", "ddd_toolkit"), ("facade", "renamed_facade"),
        ]:
            with self.subTest(kind=kind, alias=alias):
                if kind == "core":
                    deps = dependency(alias, "ddd-toolkit-core", ROOT / "crates/core")
                    deps += dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
                    macros = "macros"
                else:
                    deps = dependency(alias, "ddd-toolkit", ROOT)
                    macros = alias
                self.check_consumer(deps, PLAIN_SOURCE.replace("api::", f"{alias}::").replace("macros::", f"{macros}::"))

    def test_serde_feature_boundary(self):
        # Each case is a separate workspace: macro dev-dependencies cannot
        # silently enable core's serde feature here.
        for kind in ("core", "facade"):
            for serde_enabled in (False, True):
                with self.subTest(kind=kind, serde=serde_enabled):
                    features = ["serde"] if serde_enabled else []
                    if kind == "core":
                        deps = dependency("api", "ddd-toolkit-core", ROOT / "crates/core", features)
                        deps += dependency("macros", "ddd-toolkit-macro", ROOT / "crates/macro")
                        macros = "macros"
                        if serde_enabled:
                            deps += 'serde = { version = "1", features = ["derive"] }\n'
                    else:
                        deps = dependency("api", "ddd-toolkit", ROOT, features)
                        macros = "api"
                    plain = f'#[derive({macros}::ValueObject)]\nstruct Plain(u64);\n'
                    self.check_consumer(deps, plain)
                    source = plain + f'#[derive({macros}::ValueObject)]\n#[value_object(serde)]\nstruct Id(u64);'
                    self.check_consumer(
                        deps, source,
                        error=None if serde_enabled else (
                            'cannot find `serde` in `__private`',
                            'the item is gated behind the `serde` feature',
                        ),
                    )

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
                source += SOURCE.replace("#![allow(dead_code)]", "").replace("api::", f"{name}::")
                # Test the same derive in library, integration, and doctest targets.
                source += '\n/// ```\n/// #[derive(macros::ValueObject)]\n/// struct Id(u64);\n/// ```\npub struct DocExample;\n'
                self.check_consumer(deps, source, package=package,
                                    integration=SOURCE.replace("api::", f"{name}::"))


if __name__ == "__main__":
    unittest.main()
