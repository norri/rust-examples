use std::env;

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    fn from_str(env: &str) -> Self {
        match env {
            "production" => Environment::Production,
            _ => Environment::Local,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Environment::Local)
    }
}

pub struct Config {
    pub database_url: Option<String>,
    pub database_max_connections: u32,
    pub port: String,
    pub log_level: String,
    pub credentials: Vec<(String, String)>,
    pub environment: Environment,
}

impl Config {
    pub fn new() -> Self {
        Config {
            credentials: env::var("CREDENTIALS")
                .unwrap_or_else(|_| "".to_string())
                .split(',')
                .filter_map(|cred| {
                    let mut parts = cred.split(':');
                    if let (Some(user), Some(pass)) = (parts.next(), parts.next()) {
                        Some((user.to_string(), pass.to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            database_url: env::var("DATABASE_URL").ok(),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("DATABASE_MAX_CONNECTIONS must be a number"),
            environment: Environment::from_str(
                &env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string()),
            ),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "debug".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_new_with_env_vars() {
        env::set_var("DATABASE_URL", "postgres://localhost/test");
        env::set_var("PORT", "4000");
        env::set_var("LOG_LEVEL", "info");
        env::set_var("ENVIRONMENT", "production");

        let config = Config::new();

        assert_eq!(
            config.database_url,
            Some(String::from("postgres://localhost/test"))
        );
        assert_eq!(config.port, "4000");
        assert_eq!(config.log_level, "info");
        assert!(matches!(config.environment, Environment::Production));

        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn test_config_new_without_env_vars() {
        env::remove_var("DATABASE_URL");
        env::remove_var("PORT");
        env::remove_var("LOG_LEVEL");
        env::remove_var("ENVIRONMENT");

        let config = Config::new();

        assert_eq!(config.database_url, None);
        assert_eq!(config.port, "3000");
        assert_eq!(config.log_level, "debug");
        assert!(matches!(config.environment, Environment::Local));
    }
}
