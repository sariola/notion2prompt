// src/api/connection_pool.rs
//! Connection pooling for efficient HTTP client usage.
//!
//! This module provides connection pooling and client management
//! for optimized API performance.

use crate::error::AppError;
use crate::types::ApiKey;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::{Client, ClientBuilder};
use std::sync::Arc;
use std::time::Duration;

/// Global connection pool for HTTP clients
static CONNECTION_POOL: Lazy<ConnectionPool> = Lazy::new(ConnectionPool::new);

/// Connection pool configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PoolConfig {
    /// Maximum idle connections per host
    pub max_idle_per_host: usize,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Pool timeout
    pub pool_timeout: Duration,
    /// Maximum connections
    pub max_connections: usize,
    /// Enable HTTP/2
    pub http2_prior_knowledge: bool,
    /// Enable connection keep-alive
    pub tcp_keepalive: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_host: 10,
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            pool_timeout: Duration::from_secs(90),
            max_connections: 100,
            http2_prior_knowledge: true,
            tcp_keepalive: Some(Duration::from_secs(60)),
        }
    }
}

/// Connection pool for managing HTTP clients
#[allow(dead_code)]
pub struct ConnectionPool {
    /// Default client for general use
    default_client: Arc<Client>,
    /// Specialized clients by configuration
    specialized_clients: RwLock<std::collections::HashMap<String, Arc<Client>>>,
    /// Pool configuration
    config: PoolConfig,
}

impl ConnectionPool {
    /// Create a new connection pool
    fn new() -> Self {
        let config = PoolConfig::default();
        let default_client = Arc::new(Self::build_client(&config).expect("ConnectionPool: Failed to build default HTTP client. This is a critical error - check network configuration."));

        Self {
            default_client,
            specialized_clients: RwLock::new(std::collections::HashMap::new()),
            config,
        }
    }

    /// Build a client with the given configuration
    fn build_client(config: &PoolConfig) -> Result<Client, reqwest::Error> {
        let mut builder = ClientBuilder::new()
            .pool_max_idle_per_host(config.max_idle_per_host)
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .pool_idle_timeout(config.pool_timeout);

        if config.http2_prior_knowledge {
            builder = builder.http2_prior_knowledge();
        }

        if let Some(keepalive) = config.tcp_keepalive {
            builder = builder.tcp_keepalive(keepalive);
        }

        builder.build()
    }

    /// Get the default client
    #[allow(dead_code)]
    pub fn default_client() -> Arc<Client> {
        CONNECTION_POOL.default_client.clone()
    }

    /// Get or create a specialized client
    pub fn get_or_create_client(key: &str, config: PoolConfig) -> Result<Arc<Client>, AppError> {
        let pool = &CONNECTION_POOL;

        // Check if client already exists
        {
            let clients = pool.specialized_clients.read();
            if let Some(client) = clients.get(key) {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client = Arc::new(Self::build_client(&config)?);

        // Store in pool
        {
            let mut clients = pool.specialized_clients.write();
            clients.insert(key.to_string(), client.clone());
        }

        Ok(client)
    }

    /// Clear all specialized clients
    #[allow(dead_code)]
    pub fn clear_specialized() {
        let pool = &CONNECTION_POOL;
        pool.specialized_clients.write().clear();
    }

    /// Get pool statistics
    #[allow(dead_code)]
    pub fn stats() -> PoolStats {
        let pool = &CONNECTION_POOL;
        let specialized_count = pool.specialized_clients.read().len();

        PoolStats {
            specialized_clients: specialized_count,
            config: pool.config.clone(),
        }
    }
}

/// Pool statistics
#[derive(Debug)]
#[allow(dead_code)]
pub struct PoolStats {
    pub specialized_clients: usize,
    pub config: PoolConfig,
}

/// Notion-specific HTTP client with connection pooling
#[allow(dead_code)]
pub struct PooledNotionClient {
    client: Arc<Client>,
    api_key: ApiKey,
    base_url: String,
}

impl PooledNotionClient {
    /// Create a new pooled Notion client
    #[allow(dead_code)]
    pub fn new(api_key: ApiKey) -> Result<Self, AppError> {
        let client = ConnectionPool::default_client();

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.notion.com/v1".to_string(),
        })
    }

    /// Create with custom configuration
    #[allow(dead_code)]
    pub fn with_config(api_key: ApiKey, config: PoolConfig) -> Result<Self, AppError> {
        let client = ConnectionPool::get_or_create_client("notion_custom", config)?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.notion.com/v1".to_string(),
        })
    }

    /// Get the underlying client
    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Build a request with common headers
    #[allow(dead_code)]
    pub fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path);

        self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.api_key.as_str()))
            .header("Notion-Version", "2022-06-28")
            .header("Content-Type", "application/json")
    }
}

/// Connection health monitoring
pub mod health {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Health metrics for connection monitoring
    pub struct HealthMetrics {
        successful_requests: AtomicU64,
        failed_requests: AtomicU64,
        total_latency_ms: AtomicU64,
        connection_errors: AtomicU64,
    }

    impl HealthMetrics {
        pub fn new() -> Self {
            Self {
                successful_requests: AtomicU64::new(0),
                failed_requests: AtomicU64::new(0),
                total_latency_ms: AtomicU64::new(0),
                connection_errors: AtomicU64::new(0),
            }
        }

        #[allow(dead_code)]
        pub fn record_success(&self, latency_ms: u64) {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
            self.total_latency_ms
                .fetch_add(latency_ms, Ordering::Relaxed);
        }

        #[allow(dead_code)]
        pub fn record_failure(&self) {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        pub fn record_connection_error(&self) {
            self.connection_errors.fetch_add(1, Ordering::Relaxed);
        }

        pub fn get_stats(&self) -> HealthStats {
            let successful = self.successful_requests.load(Ordering::Relaxed);
            let failed = self.failed_requests.load(Ordering::Relaxed);
            let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
            let connection_errors = self.connection_errors.load(Ordering::Relaxed);

            let avg_latency = total_latency.checked_div(successful).unwrap_or(0);

            HealthStats {
                successful_requests: successful,
                failed_requests: failed,
                average_latency_ms: avg_latency,
                connection_errors,
                success_rate: if successful + failed > 0 {
                    (successful as f64) / ((successful + failed) as f64)
                } else {
                    0.0
                },
            }
        }
    }

    #[derive(Debug)]
    pub struct HealthStats {
        #[allow(dead_code)]
        pub successful_requests: u64,
        #[allow(dead_code)]
        pub failed_requests: u64,
        #[allow(dead_code)]
        pub average_latency_ms: u64,
        #[allow(dead_code)]
        pub connection_errors: u64,
        #[allow(dead_code)]
        pub success_rate: f64,
    }

    /// Global health metrics
    static HEALTH_METRICS: Lazy<HealthMetrics> = Lazy::new(HealthMetrics::new);

    /// Record a successful request
    #[allow(dead_code)]
    pub fn record_success(latency_ms: u64) {
        HEALTH_METRICS.record_success(latency_ms);
    }

    /// Record a failed request
    #[allow(dead_code)]
    pub fn record_failure() {
        HEALTH_METRICS.record_failure();
    }

    /// Record a connection error
    #[allow(dead_code)]
    pub fn record_connection_error() {
        HEALTH_METRICS.record_connection_error();
    }

    /// Get current health statistics
    #[allow(dead_code)]
    pub fn get_health_stats() -> HealthStats {
        HEALTH_METRICS.get_stats()
    }
}

/// Retry configuration for resilient requests
#[derive(Debug, Clone)]
pub struct RetryConfig {
    #[allow(dead_code)]
    pub max_retries: u32,
    #[allow(dead_code)]
    pub initial_backoff_ms: u64,
    #[allow(dead_code)]
    pub max_backoff_ms: u64,
    #[allow(dead_code)]
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10_000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute a request with retry logic
#[allow(dead_code)]
pub async fn execute_with_retry<F, T>(f: F, config: &RetryConfig) -> Result<T, AppError>
where
    F: Fn() -> futures::future::BoxFuture<'static, Result<T, AppError>>,
{
    let mut backoff_ms = config.initial_backoff_ms;
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            backoff_ms = (backoff_ms as f64 * config.backoff_multiplier) as u64;
            backoff_ms = backoff_ms.min(config.max_backoff_ms);
        }

        match f().await {
            Ok(result) => {
                health::record_success(0); // TODO: Track actual latency
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_retries {
                    log::warn!("Request failed (attempt {}), retrying...", attempt + 1);
                }
            }
        }
    }

    health::record_failure();
    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_idle_per_host, 10);
        assert_eq!(config.max_connections, 100);
        assert!(config.http2_prior_knowledge);
    }

    #[test]
    fn test_connection_pool_default_client() {
        let client1 = ConnectionPool::default_client();
        let client2 = ConnectionPool::default_client();
        assert!(Arc::ptr_eq(&client1, &client2));
    }

    #[tokio::test]
    async fn test_retry_config() {
        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
            backoff_multiplier: 2.0,
        };

        use std::sync::atomic::{AtomicU32, Ordering};
        let attempts = AtomicU32::new(0);

        let result = execute_with_retry(
            || {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count < 2 {
                        Err(AppError::MissingConfiguration("Test error".to_string()))
                    } else {
                        Ok(42)
                    }
                })
            },
            &config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_health_metrics() {
        let metrics = health::HealthMetrics::new();

        metrics.record_success(100);
        metrics.record_success(200);
        metrics.record_failure();

        let stats = metrics.get_stats();
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.average_latency_ms, 150);
        assert!((stats.success_rate - 0.666).abs() < 0.01);
    }
}
