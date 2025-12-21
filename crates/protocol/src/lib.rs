use nom::{
    Finish, Parser, bytes::complete::take_while1, combinator::all_consuming,
    error::Error as NomError,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};
use std::{fmt, ops::Deref, str::FromStr};
use thiserror::Error;

pub const MAX_SECRET_ID_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecretId(String);

impl SecretId {
    pub fn parse(input: &str) -> Result<Self, SecretIdError> {
        let len = input.len();
        if len == 0 || len > MAX_SECRET_ID_LEN {
            return Err(SecretIdError::InvalidLength { len });
        }
        let valid = |c: char| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-');
        let mut parser = all_consuming::<_, _, NomError<_>, _>(take_while1(valid));
        parser
            .parse(input)
            .finish()
            .map(|(_, matched)| Self(matched.to_owned()))
            .map_err(|_| SecretIdError::InvalidCharacter)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for SecretId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Deref for SecretId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl FromStr for SecretId {
    type Err = SecretIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for SecretId {
    type Error = SecretIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SecretId::parse(&value)
    }
}

impl Serialize for SecretId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SecretId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SecretId::parse(&s).map_err(DeError::custom)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SecretIdError {
    #[error("secret id length must be between 1 and 64 characters (got {len})")]
    InvalidLength { len: usize },
    #[error("secret id may contain only ASCII letters, numbers, underscores, or hyphens")]
    InvalidCharacter,
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
    fn secret_id_validation() {
        assert!(SecretId::parse("valid_ID-123").is_ok());
        assert!(matches!(
            SecretId::parse(""),
            Err(SecretIdError::InvalidLength { .. })
        ));
        assert!(matches!(
            SecretId::parse("invalid!"),
            Err(SecretIdError::InvalidCharacter)
        ));
        assert!(matches!(
            SecretId::parse(&"a".repeat(MAX_SECRET_ID_LEN + 1)),
            Err(SecretIdError::InvalidLength { .. })
        ));
    }

    #[test]
    fn prost_structs_exist() {
        let request = crate::pb::ReadSecretRequest {
            id: "github_token".into(),
            nonce: "abc".into(),
        };
        assert_eq!(request.id, "github_token");
        assert_eq!(request.nonce, "abc");

        let response = crate::pb::ReadSecretResponse {
            value: "secret".into(),
        };
        assert_eq!(response.value, "secret");
    }
}
