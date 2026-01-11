use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_token_expiry: i64,
    pub jwt_refresh_token_expiry: i64,
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub environment: String,
    pub encryption_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let jwt_access_token_expiry = env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<i64>()?;

        let jwt_refresh_token_expiry = env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let encryption_key = env::var("ENCRYPTION_KEY")
            .expect("ENCRYPTION_KEY must be set");

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_access_token_expiry,
            jwt_refresh_token_expiry,
            server_host,
            server_port,
            cors_allowed_origins,
            environment,
            encryption_key,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
