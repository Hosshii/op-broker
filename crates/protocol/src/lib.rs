use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};
use std::{fmt, ops::Deref, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpSecretReference(String);

impl OpSecretReference {
    pub fn parse(input: &str) -> Result<Self, OpSecretReferenceError> {
        if input.trim().is_empty() {
            return Err(OpSecretReferenceError::Empty);
        }
        if !input.starts_with("op://") {
            return Err(OpSecretReferenceError::InvalidScheme);
        }
        Ok(Self(input.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for OpSecretReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Deref for OpSecretReference {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl FromStr for OpSecretReference {
    type Err = OpSecretReferenceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for OpSecretReference {
    type Error = OpSecretReferenceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        OpSecretReference::parse(&value)
    }
}

impl Serialize for OpSecretReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OpSecretReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        OpSecretReference::parse(&s).map_err(DeError::custom)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OpSecretReferenceError {
    #[error("secret reference must not be empty")]
    Empty,
    #[error("secret reference must start with op://")]
    InvalidScheme,
}

pub mod pb {
    tonic::include_proto!("opbroker");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("opbroker_descriptor");
}

pub use pb::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_secret_reference_validation() {
        assert!(OpSecretReference::parse("op://DevVault/GitHub/token").is_ok());
        assert!(matches!(
            OpSecretReference::parse(""),
            Err(OpSecretReferenceError::Empty)
        ));
        assert!(matches!(
            OpSecretReference::parse("vault/GitHub/token"),
            Err(OpSecretReferenceError::InvalidScheme)
        ));
    }

    #[test]
    fn prost_structs_exist() {
        let request = crate::pb::ReadSecretRequest {
            secret_reference: "op://DevVault/GitHub/token".into(),
            nonce: "abc".into(),
        };
        assert_eq!(request.secret_reference, "op://DevVault/GitHub/token");
        assert_eq!(request.nonce, "abc");

        let response = crate::pb::ReadSecretResponse {
            value: "secret".into(),
        };
        assert_eq!(response.value, "secret");
    }
}
