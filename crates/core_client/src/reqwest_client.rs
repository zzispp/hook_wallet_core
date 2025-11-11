//! Reqwest HTTP 客户端实现
//!
//! 本模块提供了基于 reqwest 库的 HTTP 客户端实现，支持自动重试、自定义请求头等功能。

use crate::{retry_policy, Client, ClientError, ContentType, CONTENT_TYPE};
use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, str::FromStr, time::Duration};

/// 基于 reqwest 的 HTTP 客户端
///
/// 实现了 `Client` trait，提供了完整的 HTTP GET/POST 请求功能，
/// 支持自动序列化/反序列化、重试机制和自定义请求头。
///
/// # 字段
/// - `base_url` - API 基础 URL，所有请求路径都会附加到此 URL 后面
/// - `client` - 底层的 reqwest 客户端实例
/// - `user_agent` - 可选的 User-Agent 请求头
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    /// API 基础 URL
    base_url: String,
    /// 底层 reqwest 客户端
    client: reqwest::Client,
    /// 可选的 User-Agent 头
    user_agent: Option<String>,
}

impl ReqwestClient {
    /// 创建一个新的 HTTP 客户端实例
    ///
    /// # 参数
    /// - `url` - API 基础 URL
    /// - `client` - 预配置的 reqwest 客户端
    ///
    /// # 返回值
    /// 新的 `ReqwestClient` 实例
    pub fn new(url: String, client: reqwest::Client) -> Self {
        Self {
            base_url: url,
            client,
            user_agent: None,
        }
    }

    /// 创建一个带自定义 User-Agent 的客户端实例
    ///
    /// # 参数
    /// - `url` - API 基础 URL
    /// - `client` - 预配置的 reqwest 客户端
    /// - `user_agent` - User-Agent 字符串
    ///
    /// # 返回值
    /// 新的 `ReqwestClient` 实例
    pub fn new_with_user_agent(url: String, client: reqwest::Client, user_agent: String) -> Self {
        Self {
            base_url: url,
            client,
            user_agent: Some(user_agent),
        }
    }

    /// 创建一个带重试机制的客户端实例
    ///
    /// 使用预配置的超时和重试策略创建客户端。
    ///
    /// # 参数
    /// - `url` - API 基础 URL
    /// - `timeout_secs` - 请求超时时间（秒）
    /// - `max_retries` - 最大重试次数
    ///
    /// # 返回值
    /// 新的 `ReqwestClient` 实例
    ///
    /// # Panics
    /// 如果无法构建 reqwest 客户端则会 panic
    pub fn new_with_retry(url: String, timeout_secs: u64, max_retries: u32) -> Self {
        let client = crate::client_config::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .retry(retry_policy(url.clone(), max_retries))
            .build()
            .expect("Failed to build reqwest client with retry");
        Self {
            base_url: url,
            client,
            user_agent: None,
        }
    }

    /// 创建一个用于测试的客户端实例
    ///
    /// 使用默认配置：30 秒超时，最多重试 3 次。
    ///
    /// # 参数
    /// - `url` - API 基础 URL
    ///
    /// # 返回值
    /// 新的 `ReqwestClient` 实例
    pub fn new_test_client(url: String) -> Self {
        Self::new_with_retry(url, 30, 3)
    }

    /// 设置 User-Agent 请求头
    ///
    /// # 参数
    /// - `user_agent` - User-Agent 字符串
    ///
    /// # 返回值
    /// 更新后的 `ReqwestClient` 实例（链式调用）
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// 构建完整的请求 URL
    ///
    /// 将基础 URL 和路径拼接成完整的 URL。
    ///
    /// # 参数
    /// - `path` - 请求路径（应以 `/` 开头）
    ///
    /// # 返回值
    /// 完整的 URL 字符串
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    /// 构建带自定义请求头的请求
    ///
    /// 添加 User-Agent 和其他自定义请求头到请求中。
    ///
    /// # 参数
    /// - `request` - reqwest RequestBuilder
    /// - `headers` - 可选的自定义请求头映射
    ///
    /// # 返回值
    /// 配置好请求头的 RequestBuilder
    fn build_request(&self, request: RequestBuilder, headers: Option<HashMap<String, String>>) -> RequestBuilder {
        let request = if let Some(ref user_agent) = self.user_agent {
            request.header(USER_AGENT, user_agent)
        } else {
            request
        };

        if let Some(headers) = headers {
            headers.into_iter().fold(request, |req, (key, value)| req.header(&key, &value))
        } else {
            request
        }
    }

    /// 处理 HTTP 响应并反序列化结果
    ///
    /// 读取响应体并根据状态码判断成功或失败，成功时反序列化为指定类型。
    ///
    /// # 类型参数
    /// - `R` - 响应数据类型，必须实现 `DeserializeOwned`
    ///
    /// # 参数
    /// - `response` - reqwest Response 对象
    ///
    /// # 返回值
    /// - `Ok(R)` - 反序列化后的响应数据
    /// - `Err(ClientError)` - 网络错误、HTTP 错误或反序列化错误
    async fn send_request<R>(&self, response: reqwest::Response) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let status = response.status();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| ClientError::Network(format!("Failed to read response body: {e}")))?;
        let body = String::from_utf8_lossy(&body_bytes);

        if status.is_success() {
            serde_json::from_slice(&body_bytes).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: status {} {}", status, e)))
        } else {
            Err(ClientError::Http {
                status: status.as_u16(),
                len: body.len(),
            })
        }
    }

    /// 将 reqwest 错误映射为 ClientError
    ///
    /// 根据错误类型转换为对应的 ClientError 枚举。
    ///
    /// # 参数
    /// - `e` - reqwest 错误
    ///
    /// # 返回值
    /// 对应的 `ClientError`
    fn map_reqwest_error(e: reqwest::Error) -> ClientError {
        if e.is_timeout() {
            ClientError::Timeout
        } else if e.is_connect() {
            ClientError::Network(format!("Connection error: {e}"))
        } else {
            ClientError::Network(e.to_string())
        }
    }
}

#[async_trait]
impl Client for ReqwestClient {
    async fn get<R>(&self, path: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        self.get_with_headers(path, None).await
    }

    async fn get_with_headers<R>(&self, path: &str, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let request = self.build_request(self.client.get(&url), headers);

        let response = request.send().await.map_err(Self::map_reqwest_error)?;
        self.send_request(response).await
    }

    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);
        let headers = headers.unwrap_or_else(|| HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string())]));

        let content_type = headers.get(CONTENT_TYPE).and_then(|s| ContentType::from_str(s).ok());

        let request_body = match content_type {
            Some(ContentType::TextPlain) | Some(ContentType::ApplicationFormUrlEncoded) | Some(ContentType::ApplicationXBinary) => {
                let json_value = serde_json::to_value(body).map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {e}")))?;
                match json_value {
                    serde_json::Value::String(s) => {
                        if content_type == Some(ContentType::ApplicationXBinary) {
                            // For binary content, decode hex string to bytes
                            hex::decode(&s).map_err(|e| ClientError::Serialization(format!("Failed to decode hex string: {e}")))?
                        } else {
                            s.into_bytes()
                        }
                    }
                    _ => {
                        return Err(ClientError::Serialization(
                            "Expected string body for text/plain or binary content-type".to_string(),
                        ))
                    }
                }
            }
            _ => serde_json::to_vec(body).map_err(|e| ClientError::Serialization(format!("Failed to serialize request: {e}")))?,
        };

        let request = self.build_request(self.client.post(&url).body(request_body), Some(headers));

        let response = request.send().await.map_err(Self::map_reqwest_error)?;

        self.send_request(response).await
    }
}
