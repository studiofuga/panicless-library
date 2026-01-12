use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a connector for an AI provider with encrypted token
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connector {
    pub id: i32,
    pub user_id: i32,
    pub provider: String,
    /// Encrypted token - never sent to client in full
    #[serde(skip_serializing)]
    pub encrypted_token: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response sent to client (without encrypted token details)
#[derive(Debug, Clone, Serialize)]
pub struct ConnectorResponse {
    pub id: i32,
    pub provider: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Connector> for ConnectorResponse {
    fn from(connector: Connector) -> Self {
        ConnectorResponse {
            id: connector.id,
            provider: connector.provider,
            is_active: connector.is_active,
            last_used_at: connector.last_used_at,
            created_at: connector.created_at,
            updated_at: connector.updated_at,
        }
    }
}

/// Request payload to create or update a connector
#[derive(Debug, Deserialize)]
pub struct CreateConnectorRequest {
    /// One of: "anthropic", "gemini", "chatgpt"
    pub provider: String,
    /// API token/key for the provider (plaintext, will be encrypted before storage)
    pub api_token: String,
}

/// Request payload to update a connector's active status
#[derive(Debug, Deserialize)]
pub struct UpdateConnectorRequest {
    /// Optional new API token
    pub api_token: Option<String>,
    /// Optional toggle of active status
    pub is_active: Option<bool>,
}

/// Validate that the provider is one of the supported providers
pub fn validate_provider(provider: &str) -> Result<(), String> {
    match provider {
        "anthropic" | "gemini" | "chatgpt" => Ok(()),
        _ => Err(format!(
            "Invalid provider '{}'. Must be one of: anthropic, gemini, chatgpt",
            provider
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_provider_valid() {
        assert!(validate_provider("anthropic").is_ok());
        assert!(validate_provider("gemini").is_ok());
        assert!(validate_provider("chatgpt").is_ok());
    }

    #[test]
    fn test_validate_provider_invalid() {
        assert!(validate_provider("invalid").is_err());
        assert!(validate_provider("openai").is_err());
        assert!(validate_provider("").is_err());
    }

    #[test]
    fn test_connector_to_response() {
        let connector = Connector {
            id: 1,
            user_id: 42,
            provider: "anthropic".to_string(),
            encrypted_token: "secret_encrypted_token".to_string(),
            is_active: true,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response: ConnectorResponse = connector.into();
        assert_eq!(response.id, 1);
        assert_eq!(response.provider, "anthropic");
        assert_eq!(response.is_active, true);
    }
}
