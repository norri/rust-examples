use std::env;

pub struct Config {
    pub port: String,
    pub metrics_port: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            metrics_port: env::var("METRICS_PORT").unwrap_or_else(|_| "8081".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_default_values() {
        env::remove_var("PORT");
        env::remove_var("METRICS_PORT");
        let config = Config::new();
        assert_eq!(config.port, "8080");
        assert_eq!(config.metrics_port, "8081");
    }

    #[test]
    fn test_config_custom_values() {
        env::set_var("PORT", "9090");
        env::set_var("METRICS_PORT", "9091");
        let config = Config::new();
        assert_eq!(config.port, "9090");
        assert_eq!(config.metrics_port, "9091");
        env::remove_var("PORT");
        env::remove_var("METRICS_PORT");
    }
}
