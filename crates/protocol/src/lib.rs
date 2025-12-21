use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, ops::Deref, str::FromStr};
use thiserror::Error;

pub const MAX_SECRET_ID_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecretId(String);

impl SecretId {
    pub fn parse(input: &str) -> Result<Self, SecretIdError> {
        let len = input.len();
        if len == 0 || len > MAX_SECRET_ID_LEN {
            return Err(SecretIdError::InvalidLength { len });
        }
        if !input
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'))
        {
            return Err(SecretIdError::InvalidCharacter);
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum BrokerRequest {
    Read(ReadRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadRequest {
    pub id: SecretId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrokerResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BrokerResponse {
    pub fn success(value: impl Into<String>) -> Self {
        Self {
            ok: true,
            value: Some(value.into()),
            error: None,
        }
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            value: None,
            error: Some(message.into()),
        }
    }
}

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
    fn request_roundtrip() {
        let id = SecretId::parse("github_token").unwrap();
        let request = BrokerRequest::Read(ReadRequest {
            id: id.clone(),
            nonce: Some("abc".into()),
        });
        let json = serde_json::to_string(&request).unwrap();
        let back: BrokerRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, back);
        assert_eq!(
            back,
            BrokerRequest::Read(ReadRequest {
                id,
                nonce: Some("abc".into())
            })
        );
    }

    #[test]
    fn response_helpers() {
        let ok = BrokerResponse::success("value");
        assert!(ok.ok);
        assert_eq!(ok.value.as_deref(), Some("value"));
        assert!(ok.error.is_none());

        let denied = BrokerResponse::denied("DENIED");
        assert!(!denied.ok);
        assert_eq!(denied.error.as_deref(), Some("DENIED"));
        assert!(denied.value.is_none());
    }
}
