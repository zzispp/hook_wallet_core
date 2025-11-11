use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use tracing::Level;

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// 应用名称
    #[serde(default = "default_app_name")]
    pub app_name: String,

    /// 服务器配置
    #[serde(default)]
    pub server: ServerSettings,

    /// Tracing 配置
    #[serde(default)]
    pub tracing: TracingConfig,
}

fn default_app_name() -> String {
    "app".to_string()
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,

    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

/// Tracing 配置
#[derive(Debug, Clone, Deserialize)]
pub struct TracingConfig {
    /// 日志级别 (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// 是否显示目标模块
    #[serde(default = "default_true")]
    pub with_target: bool,

    /// 是否显示线程 ID
    #[serde(default)]
    pub with_thread_ids: bool,

    /// 是否显示线程名称
    #[serde(default)]
    pub with_thread_names: bool,

    /// 是否显示文件名
    #[serde(default = "default_true")]
    pub with_file: bool,

    /// 是否显示行号
    #[serde(default = "default_true")]
    pub with_line_number: bool,

    /// 是否使用 pretty 格式
    #[serde(default)]
    pub pretty: bool,

    /// 是否使用 JSON 格式
    #[serde(default)]
    pub json: bool,

    /// 是否使用 ANSI 颜色
    #[serde(default = "default_true")]
    pub with_ansi: bool,

    /// 自定义过滤器 (例如: "my_crate=debug,other_crate=info")
    #[serde(default)]
    pub filter: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            with_target: true,
            with_thread_ids: false,
            with_thread_names: false,
            with_file: true,
            with_line_number: true,
            pretty: false,
            json: false,
            with_ansi: true,
            filter: None,
        }
    }
}

impl TracingConfig {
    /// 将字符串转换为 tracing::Level
    pub fn get_level(&self) -> Level {
        match self.level.to_uppercase().as_str() {
            "TRACE" => Level::TRACE,
            "DEBUG" => Level::DEBUG,
            "INFO" => Level::INFO,
            "WARN" => Level::WARN,
            "ERROR" => Level::ERROR,
            _ => Level::INFO,
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

impl Settings {
    /// 创建配置
    ///
    /// 配置加载优先级：
    /// 1. 默认配置
    /// 2. config/default.toml
    /// 3. config/{环境}.toml (如 config/development.toml)
    /// 4. 环境变量 (APP_ 前缀)
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // 从默认配置文件加载
            .add_source(File::with_name("config/default").required(false))
            // 从环境特定的配置文件加载
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // 从环境变量加载 (格式: APP__TRACING__LEVEL=debug)
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }

    /// 从指定的配置文件创建
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        config.try_deserialize()
    }

    /// 创建开发环境配置
    pub fn dev() -> Self {
        Self {
            app_name: "app-dev".to_string(),
            server: ServerSettings::default(),
            tracing: TracingConfig {
                level: "debug".to_string(),
                pretty: true,
                with_thread_ids: true,
                with_thread_names: true,
                ..Default::default()
            },
        }
    }

    /// 创建生产环境配置
    pub fn prod() -> Self {
        Self {
            app_name: "app-prod".to_string(),
            server: ServerSettings::default(),
            tracing: TracingConfig {
                level: "info".to_string(),
                json: true,
                with_ansi: false,
                with_file: false,
                with_line_number: false,
                ..Default::default()
            },
        }
    }
}
