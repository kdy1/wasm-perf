use anyhow::{Context, Error};
use serde::{de::DeserializeOwned, Serialize};
use std::any::type_name;

pub fn serialize_ast<T>(t: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    serde_json::to_vec(t).with_context(|| {
        format!(
            "failed to serialize `{}` using serde_json",
            type_name::<T>()
        )
    })
}

pub fn deserialize_ast<T>(bytes: &[u8]) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    serde_json::from_slice(bytes).with_context(|| {
        format!(
            "failed to deserialize `{}` using serde_json",
            type_name::<T>()
        )
    })
}
