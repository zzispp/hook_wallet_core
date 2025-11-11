//! JSON-RPC 2.0 协议类型定义
//!
//! 本模块实现了 JSON-RPC 2.0 规范的核心数据结构，包括请求、响应和错误类型。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};

/// JSON-RPC 协议版本号
pub const JSONRPC_VERSION: &str = "2.0";

/// 错误码：无效的请求
pub const ERROR_INVALID_REQUEST: i32 = -32600;

/// 错误码：方法未找到
pub const ERROR_METHOD_NOT_FOUND: i32 = -32601;

/// 错误码：无效的参数
pub const ERROR_INVALID_PARAMS: i32 = -32602;

/// 错误码：内部错误
pub const ERROR_INTERNAL_ERROR: i32 = -32603;

/// JSON-RPC 请求结构
///
/// 符合 JSON-RPC 2.0 规范的请求格式。
///
/// # 字段
/// - `jsonrpc` - 协议版本，固定为 "2.0"
/// - `id` - 请求 ID，用于匹配请求和响应
/// - `method` - 要调用的方法名称
/// - `params` - 方法参数（可以是数组或对象）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    /// 协议版本
    pub jsonrpc: &'static str,
    /// 请求 ID
    pub id: u64,
    /// 方法名称
    pub method: String,
    /// 方法参数
    pub params: Value,
}

/// JSON-RPC 请求转换 trait
///
/// 实现此 trait 的类型可以转换为标准的 `JsonRpcRequest`。
pub trait JsonRpcRequestConvert {
    /// 将自身转换为 JSON-RPC 请求
    ///
    /// # 参数
    /// - `id` - 请求 ID
    ///
    /// # 返回值
    /// 标准的 `JsonRpcRequest` 结构
    fn to_req(&self, id: u64) -> JsonRpcRequest;
}

impl JsonRpcRequest {
    /// 创建一个新的 JSON-RPC 请求
    ///
    /// # 参数
    /// - `id` - 请求 ID
    /// - `method` - 方法名称
    /// - `params` - 方法参数（JSON 值）
    ///
    /// # 返回值
    /// 新的 `JsonRpcRequest` 实例
    ///
    /// # 示例
    /// ```
    /// use core_jsonrpc::types::JsonRpcRequest;
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(1, "eth_blockNumber", json!([]));
    /// assert_eq!(request.id, 1);
    /// assert_eq!(request.method, "eth_blockNumber");
    /// ```
    pub fn new(id: u64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION,
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 错误结构
///
/// 表示 JSON-RPC 调用失败时返回的错误信息。
///
/// # 字段
/// - `code` - 错误代码（负数表示预定义错误）
/// - `message` - 错误描述信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    /// 错误代码
    pub code: i32,
    /// 错误消息
    pub message: String,
}

impl Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

impl std::error::Error for JsonRpcError {}

/// JSON-RPC 成功响应结构
///
/// 表示 JSON-RPC 调用成功时返回的响应。
///
/// # 类型参数
/// - `T` - 结果数据类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse<T> {
    /// 请求 ID（可选，与请求中的 ID 对应）
    pub id: Option<u64>,
    /// 调用结果
    pub result: T,
}

/// JSON-RPC 错误响应结构
///
/// 表示 JSON-RPC 调用失败时返回的响应。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcErrorResponse {
    /// 请求 ID（可选，与请求中的 ID 对应）
    pub id: Option<u64>,
    /// 错误信息
    pub error: JsonRpcError,
}

/// JSON-RPC 结果枚举
///
/// 表示一个 JSON-RPC 调用的结果，可能是成功或失败。
///
/// # 类型参数
/// - `T` - 成功时的结果数据类型
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    /// 成功响应
    Value(JsonRpcResponse<T>),
    /// 错误响应
    Error(JsonRpcErrorResponse),
}

impl<T> JsonRpcResult<T> {
    /// 提取结果值
    ///
    /// 将 `JsonRpcResult` 转换为标准的 `Result` 类型。
    ///
    /// # 返回值
    /// - `Ok(T)` - 调用成功，返回结果值
    /// - `Err(JsonRpcError)` - 调用失败，返回错误信息
    ///
    /// # 示例
    /// ```ignore
    /// let result: JsonRpcResult<u64> = // ... from RPC call
    /// match result.take() {
    ///     Ok(value) => println!("Got value: {}", value),
    ///     Err(err) => eprintln!("Error: {}", err),
    /// }
    /// ```
    pub fn take(self) -> Result<T, JsonRpcError> {
        match self {
            JsonRpcResult::Value(value) => Ok(value.result),
            JsonRpcResult::Error(error) => Err(error.error),
        }
    }
}

/// JSON-RPC 批量调用结果集合
///
/// 用于处理批量 JSON-RPC 调用的结果，包含多个 `JsonRpcResult`。
///
/// # 类型参数
/// - `T` - 结果数据类型
pub struct JsonRpcResults<T>(pub Vec<JsonRpcResult<T>>);

impl<T> JsonRpcResults<T> {
    /// 提取所有成功的结果
    ///
    /// 遍历所有结果，提取成功的值并忽略错误（错误会打印到 stderr）。
    ///
    /// # 返回值
    /// 包含所有成功结果值的向量
    ///
    /// # 示例
    /// ```ignore
    /// let results: JsonRpcResults<u64> = // ... from batch RPC call
    /// let successful_values = results.extract();
    /// println!("Got {} successful results", successful_values.len());
    /// ```
    pub fn extract(self) -> Vec<T> {
        let mut extracted = Vec::new();
        for (i, result) in self.0.into_iter().enumerate() {
            match result {
                JsonRpcResult::Value(response) => {
                    extracted.push(response.result);
                }
                JsonRpcResult::Error(error) => {
                    eprintln!("Batch call error for request {}: {:?}", i, error);
                    // Continue processing other results
                }
            }
        }
        extracted
    }
}

impl<T> Default for JsonRpcResults<T> {
    fn default() -> Self {
        JsonRpcResults(Vec::new())
    }
}

impl<T> From<Vec<JsonRpcResult<T>>> for JsonRpcResults<T> {
    fn from(vec: Vec<JsonRpcResult<T>>) -> Self {
        JsonRpcResults(vec)
    }
}

impl<T> IntoIterator for JsonRpcResults<T> {
    type Item = JsonRpcResult<T>;
    type IntoIter = std::vec::IntoIter<JsonRpcResult<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_request_new() {
        let request = JsonRpcRequest::new(1, "eth_blockNumber", json!([]));
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 1);
        assert_eq!(request.method, "eth_blockNumber");
        assert_eq!(request.params, json!([]));
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new(42, "test_method", json!(["arg1", "arg2"]));
        let serialized = serde_json::to_string(&request).unwrap();

        // 验证序列化后的 JSON 包含正确的字段
        assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
        assert!(serialized.contains("\"id\":42"));
        assert!(serialized.contains("\"method\":\"test_method\""));
    }

    #[test]
    fn test_jsonrpc_error_display() {
        let error = JsonRpcError {
            code: ERROR_METHOD_NOT_FOUND,
            message: "Method not found".to_string(),
        };
        assert_eq!(error.to_string(), "Method not found (-32601)");
    }

    #[test]
    fn test_jsonrpc_error_codes() {
        assert_eq!(ERROR_INVALID_REQUEST, -32600);
        assert_eq!(ERROR_METHOD_NOT_FOUND, -32601);
        assert_eq!(ERROR_INVALID_PARAMS, -32602);
        assert_eq!(ERROR_INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_jsonrpc_result_take_success() {
        let response = JsonRpcResponse {
            id: Some(1),
            result: 42u64,
        };
        let result = JsonRpcResult::Value(response);

        match result.take() {
            Ok(value) => assert_eq!(value, 42),
            Err(_) => panic!("Expected success"),
        }
    }

    #[test]
    fn test_jsonrpc_result_take_error() {
        let error_response = JsonRpcErrorResponse {
            id: Some(1),
            error: JsonRpcError {
                code: ERROR_INTERNAL_ERROR,
                message: "Internal error".to_string(),
            },
        };
        let result: JsonRpcResult<u64> = JsonRpcResult::Error(error_response);

        match result.take() {
            Ok(_) => panic!("Expected error"),
            Err(error) => {
                assert_eq!(error.code, ERROR_INTERNAL_ERROR);
                assert_eq!(error.message, "Internal error");
            }
        }
    }

    #[test]
    fn test_jsonrpc_result_serialization() {
        let response = JsonRpcResponse {
            id: Some(1),
            result: "test_result".to_string(),
        };
        let result = JsonRpcResult::Value(response);

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: JsonRpcResult<String> = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(deserialized, JsonRpcResult::Value(_)));
    }

    #[test]
    fn test_jsonrpc_results_extract() {
        let results = vec![
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(1),
                result: 10,
            }),
            JsonRpcResult::Error(JsonRpcErrorResponse {
                id: Some(2),
                error: JsonRpcError {
                    code: ERROR_INTERNAL_ERROR,
                    message: "Error".to_string(),
                },
            }),
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(3),
                result: 20,
            }),
        ];

        let batch_results = JsonRpcResults(results);
        let extracted = batch_results.extract();

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0], 10);
        assert_eq!(extracted[1], 20);
    }

    #[test]
    fn test_jsonrpc_results_default() {
        let results: JsonRpcResults<u64> = JsonRpcResults::default();
        assert_eq!(results.0.len(), 0);
    }

    #[test]
    fn test_jsonrpc_results_from_vec() {
        let vec = vec![
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(1),
                result: 42,
            }),
        ];
        let results: JsonRpcResults<i32> = JsonRpcResults::from(vec);
        assert_eq!(results.0.len(), 1);
    }

    #[test]
    fn test_jsonrpc_results_into_iter() {
        let results = JsonRpcResults(vec![
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(1),
                result: 1,
            }),
            JsonRpcResult::Value(JsonRpcResponse {
                id: Some(2),
                result: 2,
            }),
        ]);

        let mut count = 0;
        for _ in results {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_jsonrpc_version_constant() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }
}
