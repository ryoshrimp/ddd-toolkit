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
#![deny(proc_macro_derive_resolution_fallback)]
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

mod imported_types {
    use super::Id;

    #[derive(macros::ValueObject)]
    #[value_object(serde)]
    struct Pair { first: Id, second: Id }

    #[derive(macros::ValueObject)]
    #[value_object(serde)]
    struct Tuple(Id, Id);

    #[derive(macros::ValueObject)]
    #[value_object(serde)]
    enum Choice { Named { id: Id }, Newtype(Id), Tuple(Id, Id) }

    #[test]
    fn imported_types_deserialize() {
        let pair = serde_json::from_str::<Pair>(r#"{"first":1,"second":2}"#).unwrap();
        assert_eq!(pair, Pair::new(Id::new(1), Id::new(2)));
        let tuple = serde_json::from_str::<Tuple>("[1,2]").unwrap();
        assert_eq!(tuple, Tuple::new(Id::new(1), Id::new(2)));
        for value in [
            Choice::new_named(Id::new(1)),
            Choice::new_newtype(Id::new(1)),
            Choice::new_tuple(Id::new(1), Id::new(2)),
        ] {
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(serde_json::from_str::<Choice>(&json).unwrap(), value);
        }
    }
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
TRAIT_ONLY_SOURCE = '''
#![allow(dead_code)]
mod domain {
    #[derive(api::ValueObject)]
    #[value_object(trait_only)]
    pub struct Name(String);

    impl Name {
        pub fn try_new(value: String) -> Result<Self, &'static str> {
            if value.is_empty() { Err("empty") } else { Ok(Self(value)) }
        }
        pub fn as_inner(&self) -> &str { &self.0 }
        pub fn into_inner(self) -> String { self.0 }
    }
    impl std::fmt::Debug for Name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("[redacted]")
        }
    }
    impl Clone for Name {
        fn clone(&self) -> Self { Self(self.0.clone()) }
    }
    impl PartialEq for Name {
        fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
    }
    impl Eq for Name {}
    impl std::hash::Hash for Name {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            std::hash::Hash::hash(&self.0, state);
        }
    }

    // These fields deliberately implement neither Debug nor Clone nor Hash.
    pub struct Opaque;
    #[derive(api::ValueObject)]
    #[value_object(trait_only)]
    pub struct Token(Opaque);
    impl Token { pub fn checked() -> Self { Self(Opaque) } }

    #[derive(PartialEq, Eq, api::Entity)]
    #[entity(trait_only)]
    pub struct Order {
        #[entity(id)]
        key: u64,
        state: u32,
    }
    impl Order {
        pub fn new(key: u64, state: u32) -> Self { Self { key, state } }
        pub fn id(&self) -> u64 { self.key }
    }
}

#[derive(api::ValueObject)]
#[value_object(trait_only)]
struct Named<T> { value: T }
#[derive(api::ValueObject)]
#[value_object(trait_only)]
struct Pair<T>(T, u32);
#[derive(api::ValueObject)]
#[value_object(trait_only)]
struct Record<T> { value: T, count: u32 }
#[derive(api::ValueObject)]
#[value_object(trait_only)]
enum Choice<T> { Unit, Tuple(T), Named { value: T } }
#[derive(api::Entity)]
#[entity(trait_only)]
struct GenericEntity<T> where T: Eq {
    #[entity(id)]
    key: T,
}

#[test]
fn existing_api_and_policies_are_preserved() {
    use api::ValueObject;
    assert!(domain::Name::try_new(String::new()).is_err());
    let name = domain::Name::try_new("ok".into()).unwrap();
    assert_eq!(format!("{name:?}"), "[redacted]");
    assert_eq!(name.as_inner(), "ok");
    assert_eq!(ValueObject::as_inner(&name), "ok");
    assert_eq!(ValueObject::into_inner(name.clone()), "ok");
    assert_eq!(name.into_inner(), "ok");
    let _ = ValueObject::into_inner(domain::Token::checked());
    assert_eq!(*ValueObject::as_inner(&Named { value: 3 }), 3);
    assert_eq!(ValueObject::into_inner(Pair(3, 4)).1, 4);
    assert_eq!(ValueObject::as_inner(&Record { value: 3, count: 4 }).count, 4);
    assert!(matches!(ValueObject::into_inner(Choice::Tuple(3)), Choice::Tuple(3)));
    let left = domain::Order::new(7, 1);
    let right = domain::Order::new(7, 2);
    assert_eq!(left.id(), 7);
    assert_eq!(api::Entity::id(&left), &7);
    assert!(left != right); // Same ID, different state: structural comparison remains.
    assert_eq!(api::Entity::id(&GenericEntity { key: 3 }), &3);
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

    def test_trait_only_facade_consumer(self):
        for alias in ("ddd_toolkit", "renamed_facade"):
            with self.subTest(alias=alias):
                deps = dependency(alias, "ddd-toolkit", ROOT)
                source = TRAIT_ONLY_SOURCE.replace("api::", f"{alias}::")
                self.check_consumer(deps, source)

    def test_trait_only_does_not_add_public_construction_or_standard_traits(self):
        deps = dependency("api", "ddd-toolkit", ROOT)
        for expression, diagnostic in [
            ('domain::Name::new(String::new())', ['error[E0599]', '`new`']),
            ('domain::Token::new(domain::Opaque)', ['error[E0599]', '`new`']),
            ('domain::Token::checked().clone()', 'no method named `clone`'),
            ('format!("{:?}", domain::Token::checked())', "doesn't implement `Debug`"),
            ('domain::Order::new(1, 2).clone()', 'no method named `clone`'),
            ('format!("{:?}", domain::Order::new(1, 2))', "doesn't implement `Debug`"),
        ]:
            with self.subTest(expression=expression):
                source = TRAIT_ONLY_SOURCE + f'\nfn forbidden() {{ let _ = {expression}; }}\n'
                self.check_consumer(deps, source, error=diagnostic)

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
            # Explicit [[example]] targets must exist when Cargo parses the facade.
            shutil.copytree(ROOT / "examples", facade / "examples")
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
                # Keep all crate-level attributes before the self-alias items.
                self_alias = f'extern crate self as {name};\npub use implementation::*;\n'
                source = SOURCE.replace("use api::", self_alias + "use api::", 1)
                source = source.replace("api::", f"{name}::")
                # Test the same derive in library, integration, and doctest targets.
                source += '\n/// ```\n/// #[derive(macros::ValueObject)]\n/// struct Id(u64);\n/// ```\npub struct DocExample;\n'
                self.check_consumer(deps, source, package=package,
                                    integration=SOURCE.replace("api::", f"{name}::"))


if __name__ == "__main__":
    unittest.main()
