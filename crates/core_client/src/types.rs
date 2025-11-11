//! 客户端错误类型定义
//!
//! 本模块定义了 HTTP 客户端可能遇到的各种错误类型。

use std::fmt;

/// HTTP 客户端错误枚举
///
/// 包含了所有可能在 HTTP 请求过程中发生的错误类型。
///
/// # 错误类型
/// - `Network` - 网络连接错误
/// - `Timeout` - 请求超时
/// - `Http` - HTTP 响应错误（非 2xx 状态码）
/// - `Serialization` - 序列化/反序列化错误
#[derive(Debug)]
pub enum ClientError {
    /// 网络连接错误，包含错误描述信息
    Network(String),
    /// 请求超时错误
    Timeout,
    /// HTTP 响应错误，包含状态码和响应体长度
    Http {
        /// HTTP 状态码
        status: u16,
        /// 响应体字节长度
        len: usize,
    },
    /// 序列化或反序列化错误，包含错误描述信息
    Serialization(String),
}

impl fmt::Display for ClientError {
    /// 格式化错误信息用于显示
    ///
    /// 将错误转换为人类可读的字符串格式。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Timeout => write!(f, "Timeout error"),
            Self::Http { status, len } => write!(f, "HTTP error: status {}, body len: {}", status, len),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<serde_json::Error> for ClientError {
    /// 将 serde_json 错误转换为 ClientError
    ///
    /// 自动将 JSON 序列化/反序列化错误转换为 ClientError::Serialization。
    ///
    /// # 参数
    /// - `err` - serde_json 错误
    ///
    /// # 返回值
    /// 包含错误信息的 ClientError::Serialization
    fn from(err: serde_json::Error) -> Self {
        ClientError::Serialization(format!("JSON error: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_display() {
        let err = ClientError::Network("Connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: Connection refused");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = ClientError::Timeout;
        assert_eq!(err.to_string(), "Timeout error");
    }

    #[test]
    fn test_http_error_display() {
        let err = ClientError::Http {
            status: 404,
            len: 123,
        };
        assert_eq!(err.to_string(), "HTTP error: status 404, body len: 123");
    }

    #[test]
    fn test_serialization_error_display() {
        let err = ClientError::Serialization("Invalid JSON".to_string());
        assert_eq!(err.to_string(), "Serialization error: Invalid JSON");
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let client_err = ClientError::from(json_err);

        match client_err {
            ClientError::Serialization(msg) => {
                assert!(msg.starts_with("JSON error:"));
            }
            _ => panic!("Expected Serialization error"),
        }
    }

    #[test]
    fn test_error_trait() {
        let err = ClientError::Network("test".to_string());
        // 验证实现了 std::error::Error trait
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_debug_format() {
        let err = ClientError::Timeout;
        let debug_str = format!("{:?}", err);
        assert_eq!(debug_str, "Timeout");
    }
}
