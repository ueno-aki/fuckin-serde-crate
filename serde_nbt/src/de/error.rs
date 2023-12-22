use serde::de;
use thiserror::Error;

use crate::nbt_types::NBTTypes;

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("MissMatch Type: {0} , {1}")]
    MissMatch(NBTTypes, NBTTypes),
    #[error("Unsupported: {0}")]
    Unsupported(String),
    #[error("DeserializeError:{0}")]
    Message(String),
}
impl de::Error for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        DeserializeError::Message(msg.to_string())
    }
}
