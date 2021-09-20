use serde::Deserialize;
use serde_derive::Deserialize;
use std::collections::BTreeMap as Map;
use std::iter::FromIterator;

fn assert_ignored<'de, T>(json: &'de str, expected: &[(&str, &str)]) -> T
where
    T: Deserialize<'de>,
{
    let de = &mut serde_json::Deserializer::from_str(json);

    let mut unused = Map::new();

    let value: T = serde_ignored::deserialize(de, |path, v: serde_json::Value| {
        unused.insert(path.to_string(), v);
    })
    .unwrap();

    let expected = Map::from_iter(
        expected
            .into_iter()
            .cloned()
            .map(|(k, v)| (k.to_owned(), serde_json::from_str(v).unwrap())),
    );
    assert_eq!(unused, expected);

    value
}

#[derive(Debug, Deserialize)]
struct V {
    used: (),
}

#[test]
fn test_readme() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Package {
        name: String,
        dependencies: Map<String, Dependency>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    struct Dependency {
        version: String,
    }

    let json = r#"{
        "name": "demo",
        "dependencies": {
            "serde": {
                "version": "1.0",
                "typo1": ""
            }
        },
        "typo2": {
            "inner": ""
        },
        "typo3": {}
    }"#;

    let ignored = &[
        ("dependencies.serde.typo1", r#""""#),
        ("typo2", r#"{"inner": ""}"#),
        ("typo3", r#"{}"#),
    ];
    let p: Package = assert_ignored(json, ignored);

    let expected = Package {
        name: "demo".to_owned(),
        dependencies: {
            let mut map = Map::new();
            map.insert(
                "serde".to_owned(),
                Dependency {
                    version: "1.0".to_owned(),
                },
            );
            map
        },
    };
    assert_eq!(p, expected);
}

#[test]
fn test_int_key() {
    #[derive(Debug, Deserialize)]
    struct Test {
        a: Map<usize, V>,
    }

    let json = r#"{
        "a": {
            "2": {
                "used": null,
                "unused": null
            }
        }
    }"#;

    let ignored = &[("a.2.unused", "null")];
    assert_ignored::<Test>(json, ignored);
}

#[test]
fn test_newtype_key() {
    type Test = Map<Key, V>;

    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct Key(&'static str);

    let json = r#"{
        "k": {
            "used": null,
            "unused": null
        }
    }"#;

    let ignored = &[("k.unused", "null")];
    assert_ignored::<Test>(json, ignored);
}

#[test]
fn test_unit_variant_key() {
    type Test = Map<Key, V>;

    #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    enum Key {
        First,
        Second,
    }

    let json = r#"{
        "First": {
            "used": null,
            "unused": null
        }
    }"#;

    let ignored = &[("First.unused", "null")];
    assert_ignored::<Test>(json, ignored);
}
