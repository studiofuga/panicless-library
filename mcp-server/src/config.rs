use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub user_id: i32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let user_id = env::var("USER_ID")
            .expect("USER_ID must be set (e.g., USER_ID=1 to access user's data)")
            .parse::<i32>()?;

        Ok(Config {
            database_url,
            user_id,
        })
    }
}
