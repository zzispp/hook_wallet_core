//! HTTP Content-Type 头部的类型定义和转换
//!
//! 本模块提供了常见的 Content-Type 枚举类型，以及与字符串之间的转换功能。

use std::str::FromStr;

/// HTTP Content-Type 头部的常量名称
pub const CONTENT_TYPE: &str = "Content-Type";

/// JSON 内容类型
const APPLICATION_JSON: &str = "application/json";

/// 纯文本内容类型
const TEXT_PLAIN: &str = "text/plain";

/// 表单 URL 编码内容类型
const APPLICATION_FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";

/// 二进制内容类型
const APPLICATION_X_BINARY: &str = "application/x-binary";

/// HTTP Content-Type 枚举
///
/// 支持常见的 HTTP 内容类型，用于在请求和响应中指定数据格式。
///
/// # 支持的类型
/// - `ApplicationJson` - JSON 格式 (application/json)
/// - `TextPlain` - 纯文本格式 (text/plain)
/// - `ApplicationFormUrlEncoded` - 表单 URL 编码 (application/x-www-form-urlencoded)
/// - `ApplicationXBinary` - 二进制格式 (application/x-binary)
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// JSON 格式 (application/json)
    ApplicationJson,
    /// 纯文本格式 (text/plain)
    TextPlain,
    /// 表单 URL 编码 (application/x-www-form-urlencoded)
    ApplicationFormUrlEncoded,
    /// 二进制格式 (application/x-binary)
    ApplicationXBinary,
}

impl ContentType {
    /// 将 ContentType 枚举转换为对应的字符串常量
    ///
    /// # 返回值
    /// 返回该内容类型对应的标准 MIME 类型字符串
    ///
    /// # 示例
    /// ```
    /// use core_client::ContentType;
    ///
    /// assert_eq!(ContentType::ApplicationJson.as_str(), "application/json");
    /// assert_eq!(ContentType::TextPlain.as_str(), "text/plain");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => APPLICATION_JSON,
            ContentType::TextPlain => TEXT_PLAIN,
            ContentType::ApplicationFormUrlEncoded => APPLICATION_FORM_URL_ENCODED,
            ContentType::ApplicationXBinary => APPLICATION_X_BINARY,
        }
    }
}

impl FromStr for ContentType {
    type Err = &'static str;

    /// 从字符串解析 ContentType 枚举
    ///
    /// # 参数
    /// - `s` - MIME 类型字符串
    ///
    /// # 返回值
    /// - `Ok(ContentType)` - 识别的内容类型
    /// - `Err(&str)` - 未知的内容类型错误信息
    ///
    /// # 示例
    /// ```
    /// use core_client::ContentType;
    /// use std::str::FromStr;
    ///
    /// let content_type = ContentType::from_str("application/json").unwrap();
    /// assert_eq!(content_type, ContentType::ApplicationJson);
    ///
    /// assert!(ContentType::from_str("unknown/type").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            APPLICATION_JSON => Ok(ContentType::ApplicationJson),
            TEXT_PLAIN => Ok(ContentType::TextPlain),
            APPLICATION_FORM_URL_ENCODED => Ok(ContentType::ApplicationFormUrlEncoded),
            APPLICATION_X_BINARY => Ok(ContentType::ApplicationXBinary),
            _ => Err("Unknown content type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_as_str() {
        assert_eq!(ContentType::ApplicationJson.as_str(), "application/json");
        assert_eq!(ContentType::TextPlain.as_str(), "text/plain");
        assert_eq!(
            ContentType::ApplicationFormUrlEncoded.as_str(),
            "application/x-www-form-urlencoded"
        );
        assert_eq!(ContentType::ApplicationXBinary.as_str(), "application/x-binary");
    }

    #[test]
    fn test_content_type_from_str() {
        assert_eq!(
            ContentType::from_str("application/json").unwrap(),
            ContentType::ApplicationJson
        );
        assert_eq!(ContentType::from_str("text/plain").unwrap(), ContentType::TextPlain);
        assert_eq!(
            ContentType::from_str("application/x-www-form-urlencoded").unwrap(),
            ContentType::ApplicationFormUrlEncoded
        );
        assert_eq!(
            ContentType::from_str("application/x-binary").unwrap(),
            ContentType::ApplicationXBinary
        );
    }

    #[test]
    fn test_content_type_from_str_unknown() {
        let result = ContentType::from_str("unknown/type");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unknown content type");
    }

    #[test]
    fn test_content_type_roundtrip() {
        let types = vec![
            ContentType::ApplicationJson,
            ContentType::TextPlain,
            ContentType::ApplicationFormUrlEncoded,
            ContentType::ApplicationXBinary,
        ];

        for content_type in types {
            let str_repr = content_type.as_str();
            let parsed = ContentType::from_str(str_repr).unwrap();
            assert_eq!(parsed, content_type);
        }
    }

    #[test]
    fn test_content_type_clone() {
        let ct1 = ContentType::ApplicationJson;
        let ct2 = ct1.clone();
        assert_eq!(ct1, ct2);
    }

    #[test]
    fn test_content_type_debug() {
        let ct = ContentType::ApplicationJson;
        let debug_str = format!("{:?}", ct);
        assert_eq!(debug_str, "ApplicationJson");
    }
}
