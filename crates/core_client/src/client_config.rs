//! HTTP 客户端配置构建器
//!
//! 本模块提供了预配置的 reqwest 客户端构建器，包含了合理的默认超时、连接池和压缩设置。

use std::time::Duration;

/// 创建一个预配置的 reqwest 客户端构建器
///
/// 返回一个配置好默认参数的 `ClientBuilder`，可以直接使用或进一步自定义。
///
/// # 默认配置
/// - **请求超时**: 30 秒 - 单个请求的最大等待时间
/// - **连接超时**: 15 秒 - 建立连接的最大等待时间
/// - **连接池空闲超时**: 90 秒 - 空闲连接在池中保留的时间
/// - **每个主机最大空闲连接数**: 20 - 连接池大小限制
/// - **TCP Keep-Alive**: 60 秒 - 保持连接活跃的间隔
/// - **压缩支持**: gzip, brotli, deflate - 自动处理响应压缩
///
/// # 返回值
/// 配置好的 `reqwest::ClientBuilder`，可以继续链式调用添加其他配置
///
/// # 示例
/// ```ignore
/// use core_client::client_config;
///
/// // 使用默认配置
/// let client = client_config::builder().build()?;
///
/// // 在默认配置基础上添加自定义设置
/// let client = client_config::builder()
///     .user_agent("MyApp/1.0")
///     .build()?;
/// ```
pub fn builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .tcp_keepalive(Duration::from_secs(60))
        .gzip(true)
        .brotli(true)
        .deflate(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creates_valid_client() {
        let client = builder().build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_can_be_customized() {
        let client = builder()
            .user_agent("TestAgent/1.0")
            .build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_builder_supports_additional_timeout() {
        let client = builder()
            .timeout(Duration::from_secs(60))
            .build();
        assert!(client.is_ok());
    }
}
