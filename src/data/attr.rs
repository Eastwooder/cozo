use crate::data::encode::Encoded;
use crate::data::id::{AttrId, EntityId};
use crate::data::keyword::Keyword;
use crate::data::value::Value;
use anyhow::Result;
use rmp_serde::Serializer;
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::{Display, Formatter};

#[derive(Debug, thiserror::Error)]
pub enum AttributeError {
    #[error("cannot convert {0} to {1}")]
    Conversion(String, String),
}

#[repr(u8)]
#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub(crate) enum AttributeCardinality {
    One = 1,
    Many = 2,
}

impl Display for AttributeCardinality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeCardinality::One => write!(f, "one"),
            AttributeCardinality::Many => write!(f, "many"),
        }
    }
}

impl TryFrom<&'_ str> for AttributeCardinality {
    type Error = AttributeError;
    fn try_from(value: &'_ str) -> std::result::Result<Self, Self::Error> {
        match value {
            "one" => Ok(AttributeCardinality::One),
            "many" => Ok(AttributeCardinality::Many),
            s => Err(AttributeError::Conversion(
                s.to_string(),
                "AttributeCardinality".to_string(),
            )),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub(crate) enum AttributeTyping {
    Ref = 1,
    Component = 2,
    Bool = 3,
    Int = 4,
    Float = 5,
    String = 6,
    Keyword = 7,
    Uuid = 8,
    Timestamp = 9,
    Bytes = 10,
    Tuple = 11,
}

impl Display for AttributeTyping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeTyping::Ref => write!(f, "ref"),
            AttributeTyping::Component => write!(f, "component"),
            AttributeTyping::Bool => write!(f, "bool"),
            AttributeTyping::Int => write!(f, "int"),
            AttributeTyping::Float => write!(f, "float"),
            AttributeTyping::String => write!(f, "string"),
            AttributeTyping::Keyword => write!(f, "keyword"),
            AttributeTyping::Uuid => write!(f, "uuid"),
            AttributeTyping::Timestamp => write!(f, "timestamp"),
            AttributeTyping::Bytes => write!(f, "bytes"),
            AttributeTyping::Tuple => write!(f, "tuple"),
        }
    }
}

impl TryFrom<&'_ str> for AttributeTyping {
    type Error = AttributeError;
    fn try_from(value: &'_ str) -> std::result::Result<Self, Self::Error> {
        use AttributeTyping::*;
        Ok(match value {
            "ref" => Ref,
            "component" => Component,
            "bool" => Bool,
            "int" => Int,
            "float" => Float,
            "string" => String,
            "keyword" => Keyword,
            "uuid" => Uuid,
            "timestamp" => Timestamp,
            "bytes" => Bytes,
            "tuple" => Tuple,
            s => {
                return Err(AttributeError::Conversion(
                    s.to_string(),
                    "AttributeTyping".to_string(),
                ))
            }
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TypeError {
    #[error("provided value {1} is not of type {0:?}")]
    Typing(AttributeTyping, String),
}

impl AttributeTyping {
    fn type_err(&self, val: Value) -> TypeError {
        TypeError::Typing(*self, format!("{:?}", val))
    }
    pub(crate) fn coerce_value<'a>(&self, val: Value<'a>) -> Result<Value<'a>> {
        match self {
            AttributeTyping::Ref | AttributeTyping::Component => match val {
                val @ Value::EnId(_) => Ok(val),
                Value::Int(s) if s > 0 => Ok(Value::EnId(EntityId(s as u64))),
                val => Err(self.type_err(val).into()),
            },
            AttributeTyping::Bool => {
                if matches!(val, Value::Bool(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
            AttributeTyping::Int => {
                if matches!(val, Value::Int(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
            AttributeTyping::Float => match val {
                v @ Value::Float(_) => Ok(v),
                Value::Int(i) => Ok(Value::Float((i as f64).into())),
                val => Err(self.type_err(val).into()),
            },
            AttributeTyping::String => {
                if matches!(val, Value::String(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
            AttributeTyping::Keyword => match val {
                val @ Value::Keyword(_) => Ok(val),
                Value::String(s) => Ok(Value::Keyword(Keyword::try_from(s.as_ref())?)),
                val => Err(self.type_err(val).into()),
            },
            AttributeTyping::Uuid => {
                if matches!(val, Value::Uuid(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
            AttributeTyping::Timestamp => match val {
                val @ Value::Timestamp(_) => Ok(val),
                Value::Int(i) => Ok(Value::Timestamp(i)),
                val => Err(self.type_err(val).into()),
            },
            AttributeTyping::Bytes => {
                if matches!(val, Value::Bytes(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
            AttributeTyping::Tuple => {
                if matches!(val, Value::Tuple(_)) {
                    Ok(val)
                } else {
                    Err(self.type_err(val).into())
                }
            }
        }
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub(crate) enum AttributeIndex {
    None = 0,
    Indexed = 1,
    Unique = 2,
    Identity = 3,
}

impl Display for AttributeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeIndex::None => write!(f, "none"),
            AttributeIndex::Indexed => write!(f, "index"),
            AttributeIndex::Unique => write!(f, "unique"),
            AttributeIndex::Identity => write!(f, "identity"),
        }
    }
}

impl TryFrom<&'_ str> for AttributeIndex {
    type Error = AttributeError;
    fn try_from(value: &'_ str) -> std::result::Result<Self, Self::Error> {
        use AttributeIndex::*;
        Ok(match value {
            "none" => None,
            "index" => Indexed,
            "unique" => Unique,
            "identity" => Identity,
            s => {
                return Err(AttributeError::Conversion(
                    s.to_string(),
                    "AttributeIndex".to_string(),
                ))
            }
        })
    }
}

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq, Debug, Deserialize, Serialize)]
pub(crate) struct Attribute {
    #[serde(rename = "i")]
    pub(crate) id: AttrId,
    #[serde(rename = "n")]
    pub(crate) alias: Keyword,
    #[serde(rename = "c")]
    pub(crate) cardinality: AttributeCardinality,
    #[serde(rename = "t")]
    pub(crate) val_type: AttributeTyping,
    #[serde(rename = "u")]
    pub(crate) indexing: AttributeIndex,
    #[serde(rename = "h")]
    pub(crate) with_history: bool,
}

const ATTR_VEC_SIZE: usize = 80;

impl Attribute {
    pub(crate) fn encode(&self) -> Encoded<ATTR_VEC_SIZE> {
        let mut inner = SmallVec::<[u8; ATTR_VEC_SIZE]>::new();
        self.serialize(&mut Serializer::new(&mut inner)).unwrap();
        Encoded { inner }
    }
    pub(crate) fn decode(data: &[u8]) -> Result<Self> {
        Ok(rmp_serde::from_slice(data)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::attr::{Attribute, AttributeCardinality, AttributeIndex, AttributeTyping};
    use crate::data::id::AttrId;
    use crate::data::keyword::Keyword;

    #[test]
    fn show_sizes() {
        let attr = Attribute {
            id: AttrId(0),
            alias: Keyword::try_from("01234567890123456789012/01234567890123456789012").unwrap(),
            cardinality: AttributeCardinality::One,
            val_type: AttributeTyping::Ref,
            indexing: AttributeIndex::None,
            with_history: false,
        };
        let encoded = attr.encode();
        dbg!(encoded.len());
        dbg!("01234567890123456789012".as_bytes().len());
        dbg!(Attribute::decode(&encoded).unwrap());
    }
}
