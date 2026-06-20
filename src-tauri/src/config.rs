use std::{fs, sync::OnceLock, time::Duration};

use anyhow::{Context, Result};
use serde::Deserialize;

const CONFIG_PATH: &str = "config.json";

static INSTANCE: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_cooldown_minutes")]
    pub cooldown_minutes: u64, // 冷却期：同一 Feed 源至少间隔 30 分钟才推下一条
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cooldown_minutes: default_cooldown_minutes(),
        }
    }
}

// Serde 反序列化默认值
fn default_cooldown_minutes() -> u64 {
    30
}

impl AppConfig {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(Self::load_or_default)
    }

    fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|e| {
            eprintln!("[config] {e:#}");
            Self::default()
        })
    }

    fn load() -> Result<Self> {
        Self::load_from(CONFIG_PATH)
    }

    fn load_from(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).context("failed to read config file")?;

        let cfg: AppConfig = serde_json::from_str(&content).context("invalid config format")?;

        Ok(cfg)
    }

    // 从指定路径加载配置
    pub fn cooldown(&self) -> Duration {
        Duration::from_secs(self.cooldown_minutes * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cooldown() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.cooldown_minutes, 30);
        assert_eq!(cfg.cooldown(), Duration::from_secs(30 * 60));
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"cooldown_minutes": 15}"#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.cooldown_minutes, 15);
        assert_eq!(cfg.cooldown(), Duration::from_secs(15 * 60));
    }

    #[test]
    fn test_parse_missing_field() {
        let json = r#"{}"#;
        let cfg: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.cooldown_minutes, 30);
    }

    #[test]
    fn test_load_file_not_found() {
        let result = AppConfig::load_from("nonexistent.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        fs::write("_test_invalid.json", "not json").unwrap();
        let result = AppConfig::load_from("_test_invalid.json");
        fs::remove_file("_test_invalid.json").ok();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_or_default_fallback() {
        let cfg = AppConfig::load_or_default();
        assert_eq!(cfg.cooldown_minutes, 30);
    }

    #[test]
    fn test_cooldown_conversion() {
        let mut cfg = AppConfig::default();
        cfg.cooldown_minutes = 60;
        assert_eq!(cfg.cooldown(), Duration::from_secs(3600));
    }

    #[test]
    fn test_global_singleton() {
        let a = AppConfig::global();
        let b = AppConfig::global();
        assert_eq!(a.cooldown_minutes, b.cooldown_minutes);
        assert_eq!(a as *const _, b as *const _);
    }
}
