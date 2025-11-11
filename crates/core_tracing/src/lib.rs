use settings::{Settings, TracingConfig};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Core Tracing 包装器
pub struct CoreTracing;

impl CoreTracing {
    /// 使用 Settings 初始化 tracing
    ///
    /// # Arguments
    /// * `settings` - 应用配置
    /// * `service_name` - 服务名称 (如 "api", "worker" 等)
    ///
    /// # Example
    /// ```no_run
    /// use settings::Settings;
    /// use core_tracing::CoreTracing;
    ///
    /// let settings = Settings::new().unwrap();
    /// let _tracing = CoreTracing::init(&settings, "api");
    /// ```
    pub fn init(settings: &Settings, service_name: &str) -> Self {
        let config = &settings.tracing;

        tracing::info!("Initializing tracing for service: {}", service_name);

        Self::init_with_config(config, service_name);

        Self
    }

    /// 使用 TracingConfig 初始化
    fn init_with_config(config: &TracingConfig, service_name: &str) {
        let level = config.get_level();

        // 构建环境过滤器
        let env_filter = if let Some(ref filter) = config.filter {
            EnvFilter::try_new(filter).unwrap_or_else(|_| {
                EnvFilter::default().add_directive(level.into())
            })
        } else {
            EnvFilter::from_default_env()
                .add_directive(level.into())
        };

        // 根据配置选择 JSON 或普通格式
        if config.json {
            let fmt_layer = fmt::layer()
                .json()
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_ansi(config.with_ansi);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        } else {
            let fmt_layer = fmt::layer()
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names)
                .with_file(config.with_file)
                .with_line_number(config.with_line_number)
                .with_ansi(config.with_ansi);

            if config.pretty {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt_layer.pretty())
                    .init();
            } else {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt_layer)
                    .init();
            }
        }

        tracing::info!(
            service = service_name,
            level = ?level,
            "Tracing initialized successfully"
        );
    }
}