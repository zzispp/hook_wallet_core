use async_trait::async_trait;
use core_client::{Client, ClientError, ContentType};
use primitives::Chain;
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
};

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub status: Option<u16>,
    pub data: Vec<u8>,
}

pub trait RpcClientError: Error + Send + Sync + 'static + Display + Sized {
    fn into_client_error(self) -> ClientError {
        ClientError::Network(format!("RPC provider error: {}", self))
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

impl Target {
    pub fn get(url: &str) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: None,
            body: None,
        }
    }

    pub fn post_json(url: &str, body: serde_json::Value) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: Some(HashMap::from([("Content-Type".into(), "application/json".into())])),
            body: Some(serde_json::to_vec(&body).expect("Failed to serialize JSON body")),
        }
    }

    pub fn set_cache_ttl(mut self, ttl: u64) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        if let Some(headers) = self.headers.as_mut() {
            headers.insert(X_CACHE_TTL.into(), ttl.to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl From<HttpMethod> for String {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
        .into()
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RpcClient<E> {
    base_url: String,
    provider: Arc<dyn RpcProvider<Error = E>>,
}

impl<E> RpcClient<E>
where
    E: RpcClientError,
{
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self { base_url, provider }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }
}

#[async_trait]
impl<E> Client for RpcClient<E>
where
    E: RpcClientError,
{
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
        let target = if let Some(headers) = headers {
            Target {
                url,
                method: HttpMethod::Get,
                headers: Some(headers),
                body: None,
            }
        } else {
            Target::get(&url)
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;

        serde_json::from_slice(&response.data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
    }

    async fn post<T, R>(&self, path: &str, body: &T, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = self.build_url(path);

        let mut request_headers = HashMap::from([("Content-Type".to_string(), ContentType::ApplicationJson.as_str().to_string())]);

        if let Some(provided_headers) = headers {
            request_headers.extend(provided_headers);
        }

        let content_type = request_headers.get("Content-Type").and_then(|s| ContentType::from_str(s).ok());

        let data = match content_type {
            Some(ContentType::TextPlain) | Some(ContentType::ApplicationFormUrlEncoded) => {
                let json_value = serde_json::to_value(body)?;
                match json_value {
                    serde_json::Value::String(s) => s.into_bytes(),
                    _ => return Err(ClientError::Serialization("Expected string body for text/plain content-type".to_string())),
                }
            }
            Some(ContentType::ApplicationXBinary) => {
                let json_value = serde_json::to_value(body)?;
                match json_value {
                    serde_json::Value::String(s) => hex::decode(&s).map_err(|e| ClientError::Serialization(format!("Failed to decode hex string: {e}")))?,
                    _ => return Err(ClientError::Serialization("Expected hex string body for binary content-type".to_string())),
                }
            }
            _ => serde_json::to_vec(body)?,
        };

        let target = Target {
            url,
            method: HttpMethod::Post,
            headers: Some(request_headers),
            body: Some(data),
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;

        serde_json::from_slice(&response.data).map_err(|e| ClientError::Serialization(format!("Failed to deserialize response: {e}")))
    }
}

#[async_trait]
impl<E> RpcProvider for RpcClient<E>
where
    E: RpcClientError,
{
    type Error = E;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        self.provider.request(target).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        self.provider.get_endpoint(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;
    use std::collections::HashMap;

    #[test]
    fn test_target_get() {
        let target = Target::get("https://eth-mainnet.g.alchemy.com/v2/demo");
        assert_eq!(target.url, "https://eth-mainnet.g.alchemy.com/v2/demo");
        assert_eq!(target.method, HttpMethod::Get);
        assert!(target.headers.is_none());
        assert!(target.body.is_none());
    }

    #[test]
    fn test_target_post_json() {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });
        let target = Target::post_json("https://eth-mainnet.g.alchemy.com/v2/demo", body);

        assert_eq!(target.url, "https://eth-mainnet.g.alchemy.com/v2/demo");
        assert_eq!(target.method, HttpMethod::Post);
        assert!(target.headers.is_some());
        assert!(target.body.is_some());

        let headers = target.headers.unwrap();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_target_set_cache_ttl() {
        let target = Target::get("https://example.com")
            .set_cache_ttl(300);

        let headers = target.headers.unwrap();
        assert_eq!(headers.get(X_CACHE_TTL).unwrap(), "300");
    }

    #[test]
    fn test_http_method_to_string() {
        assert_eq!(String::from(HttpMethod::Get), "GET");
        assert_eq!(String::from(HttpMethod::Post), "POST");
        assert_eq!(String::from(HttpMethod::Put), "PUT");
        assert_eq!(String::from(HttpMethod::Delete), "DELETE");
        assert_eq!(String::from(HttpMethod::Head), "HEAD");
        assert_eq!(String::from(HttpMethod::Options), "OPTIONS");
        assert_eq!(String::from(HttpMethod::Patch), "PATCH");
    }

    #[test]
    fn test_http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    // 创建一个模拟的 RpcProvider 用于测试
    #[derive(Debug)]
    struct MockProvider {
        endpoints: HashMap<String, String>,
    }

    #[derive(Debug)]
    struct MockError(String);

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockError {}

    impl MockProvider {
        fn new() -> Self {
            let mut endpoints = HashMap::new();
            endpoints.insert("ethereum".to_string(), "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());
            endpoints.insert("smartchain".to_string(), "https://bsc-dataseed.binance.org".to_string());
            endpoints.insert("arbitrum".to_string(), "https://arb1.arbitrum.io/rpc".to_string());
            endpoints.insert("polygon".to_string(), "https://polygon-rpc.com".to_string());

            Self { endpoints }
        }
    }

    #[async_trait]
    impl RpcProvider for MockProvider {
        type Error = MockError;

        async fn request(&self, _target: Target) -> Result<RpcResponse, Self::Error> {
            // 模拟响应
            Ok(RpcResponse {
                status: Some(200),
                data: b"{\"result\":\"0x1234\"}".to_vec(),
            })
        }

        fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
            self.endpoints
                .get(&chain.to_string())
                .cloned()
                .ok_or_else(|| MockError("Chain not supported".to_string()))
        }
    }

    #[test]
    fn test_mock_provider_get_endpoint() {
        let provider = MockProvider::new();

        assert_eq!(
            provider.get_endpoint(Chain::Ethereum).unwrap(),
            "https://eth-mainnet.g.alchemy.com/v2/demo"
        );
        assert_eq!(
            provider.get_endpoint(Chain::SmartChain).unwrap(),
            "https://bsc-dataseed.binance.org"
        );
        assert_eq!(
            provider.get_endpoint(Chain::Arbitrum).unwrap(),
            "https://arb1.arbitrum.io/rpc"
        );
        assert_eq!(
            provider.get_endpoint(Chain::Polygon).unwrap(),
            "https://polygon-rpc.com"
        );
    }

    #[tokio::test]
    async fn test_mock_provider_request() {
        let provider = MockProvider::new();
        let target = Target::get("https://example.com");

        let response = provider.request(target).await.unwrap();
        assert_eq!(response.status, Some(200));
        assert_eq!(response.data, b"{\"result\":\"0x1234\"}");
    }

    #[test]
    fn test_rpc_response_creation() {
        let response = RpcResponse {
            status: Some(200),
            data: vec![1, 2, 3],
        };

        assert_eq!(response.status, Some(200));
        assert_eq!(response.data, vec![1, 2, 3]);
    }
}

// 集成测试：测试真实的 ETH RPC 端点
#[cfg(all(test, feature = "reqwest"))]
mod integration_tests {
    use super::*;
    use primitives::Chain;

    // 真实的以太坊公共 RPC 提供者
    #[derive(Debug)]
    struct MockEthereumProvider;

    #[derive(Debug)]
    struct MockProviderError(String);

    impl std::fmt::Display for MockProviderError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockProviderError {}

    impl From<reqwest::Error> for MockProviderError {
        fn from(e: reqwest::Error) -> Self {
            MockProviderError(e.to_string())
        }
    }

    #[async_trait]
    impl RpcProvider for MockEthereumProvider {
        type Error = MockProviderError;

        async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
            let client = reqwest::Client::new();

            let mut request_builder = match target.method {
                HttpMethod::Get => client.get(&target.url),
                HttpMethod::Post => client.post(&target.url),
                _ => return Err(MockProviderError("Unsupported method".to_string())),
            };

            if let Some(headers) = target.headers {
                for (key, value) in headers {
                    request_builder = request_builder.header(&key, &value);
                }
            }

            if let Some(body) = target.body {
                request_builder = request_builder.body(body);
            }

            let response = request_builder.send().await?;
            let status = response.status().as_u16();
            let data = response.bytes().await?.to_vec();

            Ok(RpcResponse {
                status: Some(status),
                data,
            })
        }

        fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
            match chain {
                Chain::Ethereum => Ok("https://eth.llamarpc.com".to_string()),
                Chain::SmartChain => Ok("https://bsc-dataseed.binance.org".to_string()),
                Chain::Arbitrum => Ok("https://arb1.arbitrum.io/rpc".to_string()),
                Chain::Polygon => Ok("https://polygon-rpc.com".to_string()),
            }
        }
    }

    #[tokio::test]
    #[ignore] // 忽略此测试，除非明确运行（因为需要网络请求）
    async fn test_real_eth_rpc_get_block_number() {
        let provider = MockEthereumProvider;
        let endpoint = provider.get_endpoint(Chain::Ethereum).unwrap();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });

        let target = Target::post_json(&endpoint, body);
        let response = provider.request(target).await.unwrap();

        assert_eq!(response.status, Some(200));

        // 解析响应
        let json: serde_json::Value = serde_json::from_slice(&response.data).unwrap();
        assert!(json.get("result").is_some());

        println!("ETH Block Number Response: {}", json);
    }

    #[tokio::test]
    #[ignore] // 忽略此测试，除非明确运行
    async fn test_real_eth_rpc_get_chain_id() {
        let provider = MockEthereumProvider;
        let endpoint = provider.get_endpoint(Chain::Ethereum).unwrap();

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_chainId",
            "params": [],
            "id": 1
        });

        let target = Target::post_json(&endpoint, body);
        let response = provider.request(target).await.unwrap();

        assert_eq!(response.status, Some(200));

        let json: serde_json::Value = serde_json::from_slice(&response.data).unwrap();
        let result = json.get("result").unwrap().as_str().unwrap();

        // 以太坊主网的 chain ID 是 0x1 (十六进制)
        assert_eq!(result, "0x1");

        println!("ETH Chain ID: {}", result);
    }
}
