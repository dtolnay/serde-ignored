//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate serde;
//! extern crate serde_json;
//! extern crate serde_ignored;
//!
//! use std::collections::{BTreeSet as Set, BTreeMap as Map};
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Package {
//!     name: String,
//!     dependencies: Map<String, Dependency>,
//! }
//!
//! #[derive(Debug, PartialEq, Deserialize)]
//! struct Dependency {
//!     version: String,
//! }
//!
//! # fn try_main() -> Result<(), Box<::std::error::Error>> {
//! let j = r#"{
//!     "name": "demo",
//!     "dependencies": {
//!         "serde": {
//!             "version": "0.9",
//!             "typo1": ""
//!         }
//!     },
//!     "typo2": {
//!         "inner": ""
//!     },
//!     "typo3": {}
//! }"#;
//!
//! // Some Deserializer.
//! let jd = &mut serde_json::Deserializer::from_str(j);
//!
//! // We will build a set of paths to the unused elements.
//! let mut unused = Set::new();
//!
//! let p: Package = serde_ignored::deserialize(jd, |path| {
//!     unused.insert(path.to_string());
//! })?;
//!
//! assert_eq!(p, Package {
//!     name: "demo".to_owned(),
//!     dependencies: {
//!         let mut map = Map::new();
//!         map.insert("serde".to_owned(), Dependency {
//!             version: "0.9".to_owned(),
//!         });
//!         map
//!     },
//! });
//!
//! assert_eq!(unused, {
//!     let mut expected = Set::new();
//!     expected.insert("dependencies.serde.typo1".to_owned());
//!     expected.insert("typo2".to_owned());
//!     expected.insert("typo3".to_owned());
//!     expected
//! });
//!
//! # Ok(()) }
//! # fn main() { try_main().unwrap() }
//! ```

extern crate serde;

use std::fmt::{self, Display};
use serde::de::{self, Deserialize, DeserializeSeed, Visitor};

/// Entry point. See crate documentation for an example.
pub fn deserialize<D, F, T>(deserializer: D, callback: F) -> Result<T, D::Error>
    where D: de::Deserializer,
          F: FnMut(Path),
          T: Deserialize
{
    T::deserialize(Deserializer::new(deserializer, callback))
}

/// Deserializer adapter that invokes a callback with the path to every unused
/// field of the input.
pub struct Deserializer<'a, D, F> {
    de: D,
    callback: F,
    path: Path<'a>,
}

impl<'a, D, F> Deserializer<'a, D, F>
    where F: FnMut(Path)
{
    pub fn new(de: D, callback: F) -> Self {
        Deserializer {
            de: de,
            callback: callback,
            path: Path::Root,
        }
    }
}

/// Path to the current value in the input, like `dependencies.serde.typo1`.
pub enum Path<'a> {
    Root,
    Seq { parent: &'a Path<'a>, index: usize },
    Map { parent: &'a Path<'a>, key: String },
    Some { parent: &'a Path<'a> },
    NewtypeStruct { parent: &'a Path<'a> },
    NewtypeVariant { parent: &'a Path<'a> },
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        struct Parent<'a>(&'a Path<'a>);

        impl<'a> Display for Parent<'a> {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                match *self.0 {
                    Path::Root => Ok(()),
                    ref path => write!(formatter, "{}.", path),
                }
            }
        }

        match *self {
            Path::Root => formatter.write_str("."),
            Path::Seq { parent, index } => write!(formatter, "{}{}", Parent(parent), index),
            Path::Map { parent, ref key } => write!(formatter, "{}{}", Parent(parent), key),
            Path::Some { parent } |
            Path::NewtypeStruct { parent } |
            Path::NewtypeVariant { parent } => write!(formatter, "{}?", Parent(parent)),
        }
    }
}

/// Plain old forwarding impl except for `deserialize_ignored_any` which invokes
/// the callback.
impl<'a, D, F> de::Deserializer for Deserializer<'a, D, F>
    where D: de::Deserializer,
          F: FnMut(Path)
{
    type Error = D::Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_bool(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_u8(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_u16(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_u32(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_u64(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_i8(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_i16(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_i32(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_i64(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_f32(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_f64(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_char(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_str(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_string(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_bytes(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_byte_buf(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_option(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_unit(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_unit_struct(name, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_newtype_struct(name, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_seq(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_seq_fixed_size(len, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_tuple(len, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_tuple_struct<V>(self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_tuple_struct(name, len, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_map(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_struct(name, fields, Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_struct_field<V>(self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_struct_field(Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, D::Error>
        where V: Visitor
    {
        self.de.deserialize_enum(name,
                                 variants,
                                 Wrap::new(visitor, self.callback, &self.path))
    }

    fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, D::Error>
        where V: Visitor
    {
        (self.callback)(self.path);
        self.de.deserialize_ignored_any(visitor)
    }
}

/// Wrapper that attaches context to a `Visitor`, `SeqVisitor`, `EnumVisitor` or
/// `VariantVisitor`.
struct Wrap<'a, X, F> {
    delegate: X,
    callback: F,
    path: &'a Path<'a>,
}

impl<'a, X, F> Wrap<'a, X, F> {
    fn new(delegate: X, callback: F, path: &'a Path<'a>) -> Self {
        Wrap {
            delegate: delegate,
            callback: callback,
            path: path,
        }
    }
}

/// Forwarding impl to preserve context.
impl<'a, X, F> Visitor for Wrap<'a, X, F>
    where X: Visitor,
          F: FnMut(Path)
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.delegate.expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_bool(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i8(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i16(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i32(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i64(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u8(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u16(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u32(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u64(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_f32(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_f64(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_string(v)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_unit()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.delegate.visit_some(Deserializer {
            de: deserializer,
            callback: self.callback,
            path: Path::Some { parent: self.path },
        })
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.delegate.visit_newtype_struct(Deserializer {
            de: deserializer,
            callback: self.callback,
            path: Path::NewtypeStruct { parent: self.path },
        })
    }

    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::SeqVisitor
    {
        self.delegate.visit_seq(SeqVisitor::new(visitor, self.callback, self.path))
    }

    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::MapVisitor
    {
        self.delegate.visit_map(MapVisitor::new(visitor, self.callback, self.path))
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::EnumVisitor
    {
        self.delegate.visit_enum(Wrap::new(visitor, self.callback, self.path))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_byte_buf(v)
    }
}

/// Forwarding impl to preserve context.
impl<'a, X: 'a, F> de::EnumVisitor for Wrap<'a, X, F>
    where X: de::EnumVisitor,
          F: FnMut(Path)
{
    type Error = X::Error;
    type Variant = Wrap<'a, X::Variant, F>;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), X::Error>
        where V: DeserializeSeed
    {
        let callback = self.callback;
        let path = self.path;
        self.delegate
            .visit_variant_seed(seed)
            .map(move |(v, vis)| (v, Wrap::new(vis, callback, path)))
    }
}

/// Forwarding impl to preserve context.
impl<'a, X, F> de::VariantVisitor for Wrap<'a, X, F>
    where X: de::VariantVisitor,
          F: FnMut(Path)
{
    type Error = X::Error;

    fn visit_unit(self) -> Result<(), X::Error> {
        self.delegate.visit_unit()
    }

    fn visit_newtype_seed<T>(mut self, seed: T) -> Result<T::Value, X::Error>
        where T: DeserializeSeed
    {
        let path = Path::NewtypeVariant { parent: self.path };
        self.delegate
            .visit_newtype_seed(TrackedSeed::new(seed, &mut self.callback, path))
    }

    fn visit_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.visit_tuple(len, Wrap::new(visitor, self.callback, self.path))
    }

    fn visit_struct<V>(self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.visit_struct(fields, Wrap::new(visitor, self.callback, self.path))
    }
}

/// Seed that saves the string into the given optional during `visit_str` and
/// `visit_string`.
struct CaptureKey<'a, X> {
    delegate: X,
    key: &'a mut Option<String>,
}

impl<'a, X> CaptureKey<'a, X> {
    fn new(delegate: X, key: &'a mut Option<String>) -> Self {
        CaptureKey {
            delegate: delegate,
            key: key,
        }
    }
}

/// Forwarding impl.
impl<'a, X> DeserializeSeed for CaptureKey<'a, X>
    where X: DeserializeSeed
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<X::Value, D::Error>
        where D: de::Deserializer
    {
        self.delegate.deserialize(CaptureKey::new(deserializer, self.key))
    }
}

/// Forwarding impl.
impl<'a, X> de::Deserializer for CaptureKey<'a, X>
    where X: de::Deserializer
{
    type Error = X::Error;

    fn deserialize<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_bool(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_u8(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_u16(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_u32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_u64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_i8(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_i16(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_i32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_i64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_f32(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_f64(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_char(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_str(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_string(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_bytes(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_byte_buf(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_option(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_unit(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_unit_struct<V>(self,
                                  name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_unit_struct(name, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_newtype_struct<V>(self,
                                     name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_newtype_struct(name, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_seq(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_seq_fixed_size(len, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_tuple(len, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_tuple_struct<V>(self,
                                   name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_tuple_struct(name, len, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_map(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_struct<V>(self,
                             name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_struct(name, fields, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_struct_field<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_struct_field(CaptureKey::new(visitor, self.key))
    }

    fn deserialize_enum<V>(self,
                           name: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_enum(name, variants, CaptureKey::new(visitor, self.key))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, X::Error>
        where V: Visitor
    {
        self.delegate.deserialize_ignored_any(CaptureKey::new(visitor, self.key))
    }
}

/// Forwarding impl except `visit_str` and `visit_string` which save the string.
impl<'a, X> Visitor for CaptureKey<'a, X>
    where X: Visitor
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.delegate.expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_bool(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i8(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i16(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i32(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_i64(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u8(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u16(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u32(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_u64(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_f32(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_f64(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        *self.key = Some(v.to_owned());
        self.delegate.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: de::Error
    {
        *self.key = Some(v.clone());
        self.delegate.visit_string(v)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_unit()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.delegate.visit_some(deserializer)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.delegate.visit_newtype_struct(deserializer)
    }

    fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::SeqVisitor
    {
        self.delegate.visit_seq(visitor)
    }

    fn visit_map<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::MapVisitor
    {
        self.delegate.visit_map(visitor)
    }

    fn visit_enum<V>(self, visitor: V) -> Result<Self::Value, V::Error>
        where V: de::EnumVisitor
    {
        self.delegate.visit_enum(visitor)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: de::Error
    {
        self.delegate.visit_byte_buf(v)
    }
}

/// Seed used for map values, sequence elements and newtype variants to track
/// their path.
struct TrackedSeed<'a, X, F: 'a> {
    seed: X,
    callback: &'a mut F,
    path: Path<'a>,
}

impl<'a, X, F> TrackedSeed<'a, X, F> {
    fn new(seed: X, callback: &'a mut F, path: Path<'a>) -> Self {
        TrackedSeed {
            seed: seed,
            callback: callback,
            path: path,
        }
    }
}

impl<'a, X, F> DeserializeSeed for TrackedSeed<'a, X, F>
    where X: DeserializeSeed,
          F: FnMut(Path)
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<X::Value, D::Error>
        where D: de::Deserializer
    {
        self.seed.deserialize(Deserializer {
            de: deserializer,
            callback: self.callback,
            path: self.path,
        })
    }
}

/// Seq visitor that tracks the index of its elements.
struct SeqVisitor<'a, X, F> {
    delegate: X,
    callback: F,
    path: &'a Path<'a>,
    index: usize,
}

impl<'a, X, F> SeqVisitor<'a, X, F> {
    fn new(delegate: X, callback: F, path: &'a Path<'a>) -> Self {
        SeqVisitor {
            delegate: delegate,
            callback: callback,
            path: path,
            index: 0,
        }
    }
}

/// Forwarding impl to preserve context.
impl<'a, X, F> de::SeqVisitor for SeqVisitor<'a, X, F>
    where X: de::SeqVisitor,
          F: FnMut(Path)
{
    type Error = X::Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, X::Error>
        where T: DeserializeSeed
    {
        let path = Path::Seq {
            parent: self.path,
            index: self.index,
        };
        self.index += 1;
        self.delegate.visit_seed(TrackedSeed::new(seed, &mut self.callback, path))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.delegate.size_hint()
    }
}

/// Map visitor that captures the string value of its keys and uses that to
/// track the path to its values.
struct MapVisitor<'a, X, F> {
    delegate: X,
    callback: F,
    path: &'a Path<'a>,
    key: Option<String>,
}

impl<'a, X, F> MapVisitor<'a, X, F> {
    fn new(delegate: X, callback: F, path: &'a Path<'a>) -> Self {
        MapVisitor {
            delegate: delegate,
            callback: callback,
            path: path,
            key: None,
        }
    }

    fn key<E>(&mut self) -> Result<String, E>
        where E: de::Error
    {
        self.key.take().ok_or_else(|| E::custom("non-string key"))
    }
}

impl<'a, X, F> de::MapVisitor for MapVisitor<'a, X, F>
    where X: de::MapVisitor,
          F: FnMut(Path)
{
    type Error = X::Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, X::Error>
        where K: DeserializeSeed
    {
        self.delegate.visit_key_seed(CaptureKey::new(seed, &mut self.key))
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value, X::Error>
        where V: DeserializeSeed
    {
        let path = Path::Map {
            parent: self.path,
            key: self.key()?,
        };
        self.delegate.visit_value_seed(TrackedSeed::new(seed, &mut self.callback, path))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.delegate.size_hint()
    }
}
