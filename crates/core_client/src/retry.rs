//! HTTP 请求重试机制
//!
//! 本模块提供了 HTTP 请求的重试策略和重试逻辑，用于处理临时性网络故障和服务端错误。

use reqwest::{retry, StatusCode};
use std::future::Future;
use std::time::Duration;

#[cfg(feature = "reqwest")]
use tokio::time::sleep;

/// 为特定主机创建重试策略
///
/// 根据 HTTP 响应状态码自动判断是否应该重试请求。
///
/// # 会触发重试的情况
/// - 429 Too Many Requests - 请求过多
/// - 500 Internal Server Error - 服务器内部错误
/// - 502 Bad Gateway - 网关错误
/// - 503 Service Unavailable - 服务不可用
/// - 504 Gateway Timeout - 网关超时
/// - 网络错误（无状态码）
///
/// # 类型参数
/// - `S` - 主机标识符类型，可以与字符串切片比较
///
/// # 参数
/// - `host` - 目标主机标识
/// - `max_retries` - 最大重试次数
///
/// # 返回值
/// reqwest 重试构建器，可用于配置 HTTP 客户端
///
/// # 示例
/// ```ignore
/// let retry_builder = retry_policy("api.example.com", 3);
/// let client = reqwest::Client::builder()
///     .retry(retry_builder)
///     .build()?;
/// ```
pub fn retry_policy<S>(host: S, max_retries: u32) -> retry::Builder
where
    S: for<'a> PartialEq<&'a str> + Send + Sync + 'static,
{
    retry::for_host(host).max_retries_per_request(max_retries).classify_fn(|req_rep| {
        match req_rep.status() {
            Some(StatusCode::TOO_MANY_REQUESTS)
            | Some(StatusCode::INTERNAL_SERVER_ERROR)
            | Some(StatusCode::BAD_GATEWAY)
            | Some(StatusCode::SERVICE_UNAVAILABLE)
            | Some(StatusCode::GATEWAY_TIMEOUT) => req_rep.retryable(),
            None => req_rep.retryable(), // Network errors
            _ => req_rep.success(),
        }
    })
}

/// 带指数退避的通用重试函数
///
/// 执行异步操作，失败时使用指数退避策略自动重试。
/// 退避策略：第 n 次重试延迟 2^n 秒（2s, 4s, 8s, ...），最大延迟 30 分钟。
///
/// # 类型参数
/// - `T` - 操作成功时的返回值类型
/// - `E` - 错误类型，必须实现 `Display` trait
/// - `F` - 返回 Future 的闭包类型
/// - `Fut` - Future 类型
/// - `P` - 判断是否应该重试的谓词函数类型
///
/// # 参数
/// - `operation` - 要执行的异步操作闭包
/// - `max_retries` - 最大重试次数
/// - `should_retry_fn` - 可选的自定义重试判断函数，为 None 时使用默认判断逻辑
///
/// # 返回值
/// - `Ok(T)` - 操作成功的结果
/// - `Err(E)` - 达到最大重试次数或遇到不可重试错误后的错误
///
/// # 示例
/// ```ignore
/// use core_client::retry;
///
/// let result = retry(
///     || async { fetch_data().await },
///     3,
///     None
/// ).await?;
/// ```
pub async fn retry<T, E, F, Fut, P>(operation: F, max_retries: u32, should_retry_fn: Option<P>) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                let should_retry_error = match &should_retry_fn {
                    Some(predicate) => predicate(&err),
                    None => default_should_retry(&err),
                };

                if should_retry_error && attempt < max_retries {
                    attempt += 1;
                    // Exponential backoff: 2^attempt seconds (2s, 4s, 8s, ...) with max cap
                    let delay = Duration::from_secs(2_u64.saturating_pow(attempt).min(1800)); // Cap at 30 minutes

                    #[cfg(feature = "reqwest")]
                    sleep(delay).await;

                    #[cfg(not(feature = "reqwest"))]
                    std::thread::sleep(delay);

                    continue;
                }

                return Err(err);
            }
        }
    }
}

/// 默认的重试判断逻辑
///
/// 判断错误是否为明显的临时性错误，应该进行重试。
///
/// # 会触发重试的错误
/// - 429 (Too Many Requests) - 请求过多
/// - 502 (Bad Gateway) - 网关错误
/// - 503 (Service Unavailable) - 服务不可用
/// - 504 (Gateway Timeout) - 网关超时
/// - 包含 "too many requests" 的消息 - 限流消息
/// - 包含 "throttled" 的消息 - 节流消息
///
/// # 类型参数
/// - `E` - 错误类型，必须实现 `Display` trait
///
/// # 参数
/// - `error` - 要判断的错误引用
///
/// # 返回值
/// - `true` - 应该重试
/// - `false` - 不应该重试
///
/// # 示例
/// ```
/// use core_client::default_should_retry;
///
/// let error = "HTTP 429: Too Many Requests";
/// assert!(default_should_retry(&error));
///
/// let error = "HTTP 404: Not Found";
/// assert!(!default_should_retry(&error));
/// ```
pub fn default_should_retry<E: std::fmt::Display>(error: &E) -> bool {
    let error_str = error.to_string().to_lowercase();

    error_str.contains("429") ||                    // Too Many Requests
    error_str.contains("502") ||                    // Bad Gateway
    error_str.contains("503") ||                    // Service Unavailable
    error_str.contains("504") ||                    // Gateway Timeout
    error_str.contains("too many requests") ||      // Rate limiting messages
    error_str.contains("throttled") // Throttling messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_should_retry_429() {
        assert!(default_should_retry(&"HTTP 429 error"));
        assert!(default_should_retry(&"Error: 429"));
    }

    #[test]
    fn test_default_should_retry_502() {
        assert!(default_should_retry(&"HTTP 502 Bad Gateway"));
        assert!(default_should_retry(&"502 error occurred"));
    }

    #[test]
    fn test_default_should_retry_503() {
        assert!(default_should_retry(&"Service Unavailable 503"));
        assert!(default_should_retry(&"Error 503"));
    }

    #[test]
    fn test_default_should_retry_504() {
        assert!(default_should_retry(&"Gateway Timeout 504"));
        assert!(default_should_retry(&"504 timeout"));
    }

    #[test]
    fn test_default_should_retry_rate_limit_messages() {
        assert!(default_should_retry(&"Too many requests, please try later"));
        assert!(default_should_retry(&"Request throttled"));
    }

    #[test]
    fn test_default_should_not_retry_other_errors() {
        assert!(!default_should_retry(&"HTTP 400 Bad Request"));
        assert!(!default_should_retry(&"404 Not Found"));
        assert!(!default_should_retry(&"401 Unauthorized"));
        assert!(!default_should_retry(&"500 Internal Server Error"));
    }

    #[test]
    fn test_default_should_retry_case_insensitive() {
        assert!(default_should_retry(&"TOO MANY REQUESTS"));
        assert!(default_should_retry(&"THROTTLED"));
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry(
            move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<i32, String>(42)
                }
            },
            3,
            None::<fn(&String) -> bool>,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry(
            move || {
                let count = call_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 3 {
                        Err("Error 503".to_string())
                    } else {
                        Ok(100)
                    }
                }
            },
            5,
            None::<fn(&String) -> bool>,
        )
        .await;

        assert_eq!(result.unwrap(), 100);
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_retries_exceeded() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry(
            move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("Error 503".to_string())
                }
            },
            2,
            None::<fn(&String) -> bool>,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // Initial attempt + 2 retries
    }

    #[tokio::test]
    async fn test_retry_non_retryable_error() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry(
            move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("Error 404 Not Found".to_string())
                }
            },
            3,
            None::<fn(&String) -> bool>,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not retry
    }

    #[tokio::test]
    async fn test_retry_custom_predicate() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        let custom_predicate = |e: &String| e.contains("custom");

        let result = retry(
            move || {
                let count = call_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                    if current < 2 {
                        Err("custom error".to_string())
                    } else {
                        Ok(999)
                    }
                }
            },
            3,
            Some(custom_predicate),
        )
        .await;

        assert_eq!(result.unwrap(), 999);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
