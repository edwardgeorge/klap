#[cfg(feature = "serde_support")]
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

use crate::parser;

macro_rules! string_newtype {
    ($ty:ident) => {
        impl $ty {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $ty {
            fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::ops::Deref for $ty {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::convert::From<$ty> for String {
            fn from(val: $ty) -> Self {
                val.to_string()
            }
        }

        impl std::borrow::Borrow<str> for $ty {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl std::convert::AsRef<str> for $ty {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

macro_rules! parse_from {
    ($ty:ident, $parse_func:path) => {
        impl $ty {
            pub fn parse_str(input: &str) -> Result<$ty, Error> {
                $parse_func(input)
            }
        }

        impl std::convert::TryFrom<&str> for $ty {
            type Error = Error;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                $ty::parse_str(value)
            }
        }

        impl std::str::FromStr for $ty {
            type Err = Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $ty::parse_str(s)
            }
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    ParserError(#[from] pest::error::Error<parser::Rule>),
    //#[error("{0}")]
    //CustomError(String),
}

//#[cfg(feature="serde_support")]
//impl serde::de::Error for Error {
//    fn custom<T>(msg: T) -> Self where T: fmt::Display {
//        Error::CustomError(format!("{}", msg))
//    }
//}

#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct KeyPrefix(pub(crate) String);
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct KeyName(pub(crate) String);

string_newtype!(KeyPrefix);
string_newtype!(KeyName);
parse_from!(KeyPrefix, parser::label_keyprefix_from_str);
parse_from!(KeyName, parser::label_keyname_from_str);

/// A kubernetes label/annotation key
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
pub struct Key {
    /// An optional prefix
    prefix: Option<KeyPrefix>,
    /// The name
    name: KeyName,
}

parse_from!(Key, parser::label_key_from_str);

impl Key {
    pub fn new(prefix: Option<KeyPrefix>, name: KeyName) -> Self {
        Key { prefix, name }
    }
    pub fn new_with_prefix(prefix: KeyPrefix, name: KeyName) -> Self {
        Key {
            prefix: Some(prefix),
            name,
        }
    }
    pub fn new_no_prefix(name: KeyName) -> Self {
        Key { prefix: None, name }
    }
    pub fn prefix(&self) -> Option<&str> {
        match &self.prefix {
            Some(prefix) => Some(prefix.as_str()),
            None => None,
        }
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn without_prefix(self) -> Self {
        Key {
            prefix: None,
            name: self.name,
        }
    }
    pub fn with_prefix(self, prefix: KeyPrefix) -> Key {
        Key {
            prefix: Some(prefix),
            name: self.name,
        }
    }
    pub fn has_prefix(&self) -> bool {
        self.prefix.is_some()
    }
}

impl fmt::Display for Key {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        if let Some(ref prefix) = self.prefix {
            write!(f, "{}/", prefix)?;
        }
        write!(f, "{}", self.name)
    }
}

//LabelValue, constructed via parsing only
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
// #[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct LabelValue(pub(crate) String);

string_newtype!(LabelValue);
parse_from!(LabelValue, parser::label_value_from_str);

/// A Label is a key/value following the k8s validation rules
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct Label {
    pub key: Key,
    pub value: LabelValue,
}

impl Label {
    pub fn new(key: Key, value: LabelValue) -> Self {
        Label { key, value }
    }
    pub fn into_tuple(self) -> (Key, LabelValue) {
        (self.key, self.value)
    }
}

impl std::convert::From<(Key, LabelValue)> for Label {
    fn from(input: (Key, LabelValue)) -> Self {
        Label {
            key: input.0,
            value: input.1,
        }
    }
}

/// A Label is a key/value following the k8s validation rules
#[derive(PartialEq, Eq, Debug, Clone, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde_support", derive(Serialize))]
pub struct Annotation {
    pub key: Key,
    pub value: String,
}

impl Annotation {
    pub fn new(key: Key, value: String) -> Self {
        Annotation { key, value }
    }
    pub fn into_tuple(self) -> (Key, String) {
        (self.key, self.value)
    }
}

impl std::convert::From<(Key, String)> for Annotation {
    fn from(input: (Key, String)) -> Self {
        Annotation {
            key: input.0,
            value: input.1,
        }
    }
}

pub type Labels = Vec<Label>;
pub type LabelMap = HashMap<Key, LabelValue>;
pub type Annotations = Vec<Annotation>;
pub type AnnotationMap = HashMap<Key, String>;

#[cfg(feature = "serde_support")]
mod serde_extras {
    use serde::{de::Error, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    use super::{Key, LabelValue};
    use crate::parser::{label_key_from_str, label_value_from_str};

    macro_rules! string_newtype_visitor {
        ($ty:ident, $visitor:ident, $parse_func:ident, $expected:expr) => {
            impl<'de> Visitor<'de> for $visitor {
                type Value = $ty;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, $expected)
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    $parse_func(v).map_err(Error::custom)
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    $parse_func(&v).map_err(Error::custom)
                }

                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    $parse_func(v).map_err(Error::custom)
                }
            }
        };
    }

    macro_rules! string_newtype_instances {
        ($ty:ident, $visitor:ident) => {
            impl<'de> Deserialize<'de> for $ty {
                fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_str($visitor)
                }
            }

            impl Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serialize_with_display(self, serializer)
                }
            }
        };
    }

    struct KeyVisitor;
    struct ValueVisitor;

    string_newtype_visitor!(
        Key,
        KeyVisitor,
        label_key_from_str,
        "a valid kubernetes key (with or without prefix)"
    );
    string_newtype_visitor!(
        LabelValue,
        ValueVisitor,
        label_value_from_str,
        "a valid kubernetes label value"
    );

    pub fn serialize_with_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    //    pub fn deserialize_key<'de, D>(deser: D) -> Result<Key, D::Error>
    //    where
    //        D: Deserializer<'de>,
    //    {
    //        deser.deserialize_str(KeyVisitor)
    //    }
    //
    //    // module for serde 'with' attr
    //    pub mod with_key {
    //        pub use super::{deserialize_key as deserialize, serialize_with_display as serialize};
    //    }

    string_newtype_instances!(Key, KeyVisitor);
    string_newtype_instances!(LabelValue, ValueVisitor);
}
