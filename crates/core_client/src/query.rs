//! URL 查询参数构建工具
//!
//! 本模块提供了将结构体序列化为 URL 查询参数字符串的功能。

use serde::Serialize;

/// 从可序列化的结构体构建带查询参数的路径
///
/// 该函数接收一个基础路径和一个可序列化的结构体，
/// 将结构体字段转换为 URL 查询参数格式并附加到路径后面。
///
/// # 类型参数
/// - `T` - 实现了 `Serialize` trait 的类型
///
/// # 参数
/// - `path` - 基础 URL 路径（不含查询参数）
/// - `query` - 要序列化为查询参数的结构体引用
///
/// # 返回值
/// - `Ok(String)` - 包含查询参数的完整路径
/// - `Err(serde_urlencoded::ser::Error)` - 序列化失败时的错误
///
/// # 示例
/// ```
/// use serde::Serialize;
/// use core_client::build_path_with_query;
///
/// #[derive(Serialize)]
/// struct SearchParams {
///     q: String,
///     limit: u32,
/// }
///
/// let params = SearchParams {
///     q: "bitcoin".to_string(),
///     limit: 10,
/// };
///
/// let path = build_path_with_query("/search", &params).unwrap();
/// assert_eq!(path, "/search?q=bitcoin&limit=10");
/// ```
pub fn build_path_with_query<T: Serialize>(path: &str, query: &T) -> Result<String, serde_urlencoded::ser::Error> {
    let query_string = serde_urlencoded::to_string(query)?;
    Ok(format!("{}?{}", path, query_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct CoinQuery {
        pub market_data: bool,
        pub community_data: bool,
        pub tickers: bool,
        pub localization: bool,
        pub developer_data: bool,
    }

    #[derive(Serialize)]
    struct SimpleQuery {
        q: String,
        limit: u32,
    }

    #[test]
    fn test_build_path_with_query_coingecko_case() {
        let id = "bitcoin";
        let query = CoinQuery {
            market_data: false,
            community_data: true,
            tickers: false,
            localization: true,
            developer_data: true,
        };
        let base_path = format!("/api/v3/coins/{}", id);
        let result = build_path_with_query(&base_path, &query).unwrap();

        let expected = "/api/v3/coins/bitcoin?market_data=false&community_data=true&tickers=false&localization=true&developer_data=true";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_build_path_with_simple_query() {
        let query = SimpleQuery {
            q: "test".to_string(),
            limit: 100,
        };
        let result = build_path_with_query("/search", &query).unwrap();
        assert_eq!(result, "/search?q=test&limit=100");
    }

    #[test]
    fn test_build_path_with_empty_path() {
        let query = SimpleQuery {
            q: "hello".to_string(),
            limit: 50,
        };
        let result = build_path_with_query("", &query).unwrap();
        assert_eq!(result, "?q=hello&limit=50");
    }

    #[test]
    fn test_build_path_with_special_characters() {
        #[derive(Serialize)]
        struct SpecialQuery {
            text: String,
        }

        let query = SpecialQuery {
            text: "hello world!".to_string(),
        };
        let result = build_path_with_query("/search", &query).unwrap();
        // URL encoding: space becomes %20, ! becomes %21
        assert_eq!(result, "/search?text=hello+world%21");
    }

    #[test]
    fn test_build_path_with_optional_fields() {
        #[derive(Serialize)]
        struct OptionalQuery {
            required: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            optional: Option<String>,
        }

        let query = OptionalQuery {
            required: "value".to_string(),
            optional: None,
        };
        let result = build_path_with_query("/api", &query).unwrap();
        assert_eq!(result, "/api?required=value");

        let query_with_optional = OptionalQuery {
            required: "value".to_string(),
            optional: Some("extra".to_string()),
        };
        let result = build_path_with_query("/api", &query_with_optional).unwrap();
        assert_eq!(result, "/api?required=value&optional=extra");
    }

    #[test]
    fn test_build_path_with_numeric_types() {
        #[derive(Serialize)]
        struct NumericQuery {
            int: i32,
            uint: u64,
            float: f64,
        }

        let query = NumericQuery {
            int: -42,
            uint: 123456,
            float: 3.14,
        };
        let result = build_path_with_query("/data", &query).unwrap();
        assert_eq!(result, "/data?int=-42&uint=123456&float=3.14");
    }
}
