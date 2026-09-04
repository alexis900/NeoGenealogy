use serde::{Deserialize, Serialize};

// Provider error normalized
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderErrorCode {
    NO_RESULTS,
    PROVIDER_UNAVAILABLE,
    AUTH_REQUIRED,
    RATE_LIMITED,
    INVALID_QUERY,
    TIMEOUT,
    UNKNOWN,
}

impl ProviderErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NO_RESULTS => "NO_RESULTS",
            Self::PROVIDER_UNAVAILABLE => "PROVIDER_UNAVAILABLE",
            Self::AUTH_REQUIRED => "AUTH_REQUIRED",
            Self::RATE_LIMITED => "RATE_LIMITED",
            Self::INVALID_QUERY => "INVALID_QUERY",
            Self::TIMEOUT => "TIMEOUT",
            Self::UNKNOWN => "UNKNOWN",
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "NO_RESULTS" => Self::NO_RESULTS,
            "PROVIDER_UNAVAILABLE" => Self::PROVIDER_UNAVAILABLE,
            "AUTH_REQUIRED" => Self::AUTH_REQUIRED,
            "RATE_LIMITED" => Self::RATE_LIMITED,
            "INVALID_QUERY" => Self::INVALID_QUERY,
            "TIMEOUT" => Self::TIMEOUT,
            _ => Self::UNKNOWN,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderError {
    pub code: ProviderErrorCode,
    pub message: String,
}

impl ProviderError {
    pub fn new(code: ProviderErrorCode, msg: impl Into<String>) -> Self {
        Self {
            code,
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for ProviderError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResultCandidate {
    pub external_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub record_type: Option<String>,
    pub date: Option<String>,
    pub place: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ResearchProviderResponse {
    pub provider: String,
    pub results: Vec<ResearchResultCandidate>,
    pub provider_request_id: Option<String>,
    pub provider_metadata: serde_json::Value,
}

// Provider abstraction
#[async_trait::async_trait]
pub trait ResearchProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn search(&self, query: &str) -> Result<ResearchProviderResponse, ProviderError>;
}

// URL validation
pub fn is_valid_external_url(url: &str) -> bool {
    if url.trim().is_empty() {
        return false;
    }
    if let Ok(parsed) = url::Url::parse(url) {
        return parsed.scheme() == "http" || parsed.scheme() == "https";
    }
    false
}

// Mock provider
pub struct MockResearchProvider;

#[async_trait::async_trait]
impl ResearchProvider for MockResearchProvider {
    fn name(&self) -> &str {
        "mock"
    }
    async fn search(&self, query: &str) -> Result<ResearchProviderResponse, ProviderError> {
        let q = query.trim();
        if q.is_empty() {
            return Err(ProviderError::new(
                ProviderErrorCode::INVALID_QUERY,
                "query must not be empty",
            ));
        }
        let lower = q.to_lowercase();
        if lower.contains("fail") || lower.contains("error") {
            return Err(ProviderError::new(
                ProviderErrorCode::PROVIDER_UNAVAILABLE,
                "Mock provider is unavailable for this query",
            ));
        }
        if lower.contains("rate_limited") {
            return Err(ProviderError::new(
                ProviderErrorCode::RATE_LIMITED,
                "rate limited",
            ));
        }
        if lower.contains("timeout") {
            return Err(ProviderError::new(ProviderErrorCode::TIMEOUT, "timeout"));
        }
        if lower.contains("auth") {
            return Err(ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "auth required",
            ));
        }
        if lower.contains("no-results") || lower.contains("no_results") || lower == "empty" {
            return Ok(ResearchProviderResponse {
                provider: "mock".to_string(),
                results: vec![],
                provider_request_id: Some(format!("mock-req-{}", q.len())),
                provider_metadata: serde_json::json!({"mock": true, "query": q}),
            });
        }
        // default: 2 deterministic results
        let title1 = format!("Baptism record — {}", q);
        let title2 = format!("Census record — {}", q);
        let results = vec![
            ResearchResultCandidate {
                external_id: Some("mock-ext-1".to_string()),
                title: title1,
                description: Some(format!("Possible matching record for '{}'", q)),
                url: Some("https://example.com/record/1".to_string()),
                record_type: Some("BAPTISM".to_string()),
                date: Some("1882".to_string()),
                place: Some("Sant Martí".to_string()),
                metadata: serde_json::json!({"provider": "mock", "position": 0}),
            },
            ResearchResultCandidate {
                external_id: Some("mock-ext-2".to_string()),
                title: title2,
                description: Some(format!(
                    "Possible matching record for '{}' — second result",
                    q
                )),
                url: Some("https://example.com/record/2".to_string()),
                record_type: Some("CENSUS".to_string()),
                date: Some("1882-03-15".to_string()),
                place: Some("Sant Martí, Barcelona".to_string()),
                metadata: serde_json::json!({"provider": "mock", "position": 1}),
            },
        ];
        Ok(ResearchProviderResponse {
            provider: "mock".to_string(),
            results,
            provider_request_id: Some(format!("mock-req-{}", q.len())),
            provider_metadata: serde_json::json!({"mock": true, "query": q}),
        })
    }
}

// Registry
pub struct ResearchProviderRegistry {
    providers: std::collections::HashMap<String, std::sync::Arc<dyn ResearchProvider>>,
}

impl Default for ResearchProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchProviderRegistry {
    pub fn new() -> Self {
        let mut m: std::collections::HashMap<String, std::sync::Arc<dyn ResearchProvider>> =
            std::collections::HashMap::new();
        m.insert(
            "mock".to_string(),
            std::sync::Arc::new(MockResearchProvider) as std::sync::Arc<dyn ResearchProvider>,
        );
        // FamilySearch provider — always registered; runtime will return AUTH_REQUIRED if not configured
        let fs_config = crate::familysearch::FamilySearchConfig::from_env();
        let fs_provider = crate::familysearch::FamilySearchProvider::new(fs_config);
        m.insert(
            "familysearch".to_string(),
            std::sync::Arc::new(fs_provider) as std::sync::Arc<dyn ResearchProvider>,
        );
        Self { providers: m }
    }

    pub fn new_with_familysearch_config(config: crate::familysearch::FamilySearchConfig) -> Self {
        let mut m: std::collections::HashMap<String, std::sync::Arc<dyn ResearchProvider>> =
            std::collections::HashMap::new();
        m.insert(
            "mock".to_string(),
            std::sync::Arc::new(MockResearchProvider) as std::sync::Arc<dyn ResearchProvider>,
        );
        let fs_provider = crate::familysearch::FamilySearchProvider::new(config);
        m.insert(
            "familysearch".to_string(),
            std::sync::Arc::new(fs_provider) as std::sync::Arc<dyn ResearchProvider>,
        );
        Self { providers: m }
    }

    pub fn get(&self, name: &str) -> Option<std::sync::Arc<dyn ResearchProvider>> {
        let key = name.to_lowercase();
        self.providers.get(&key).cloned()
    }

    pub fn register(&mut self, provider: std::sync::Arc<dyn ResearchProvider>) {
        self.providers
            .insert(provider.name().to_lowercase(), provider);
    }

    pub fn available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_deterministic_results() {
        let p = MockResearchProvider;
        let r = p.search("Josep García 1882 Sant Martí").await.unwrap();
        assert_eq!(r.results.len(), 2);
        assert_eq!(r.provider, "mock");
    }

    #[tokio::test]
    async fn mock_no_results() {
        let p = MockResearchProvider;
        let r = p.search("no-results").await.unwrap();
        assert_eq!(r.results.len(), 0);
    }

    #[tokio::test]
    async fn mock_failure() {
        let p = MockResearchProvider;
        let e = p.search("fail test").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::PROVIDER_UNAVAILABLE);
    }

    #[test]
    fn url_validation() {
        assert!(is_valid_external_url("https://example.com"));
        assert!(is_valid_external_url("http://example.com/path"));
        assert!(!is_valid_external_url("javascript:alert(1)"));
        assert!(!is_valid_external_url("data:text/html,hello"));
        assert!(!is_valid_external_url("file:///etc/passwd"));
        assert!(!is_valid_external_url("ftp://example.com"));
    }

    #[test]
    fn registry_contains_mock_and_familysearch() {
        let reg = ResearchProviderRegistry::new();
        assert!(reg.get("mock").is_some());
        assert!(reg.get("familysearch").is_some());
        assert!(reg.get("Mock").is_some());
        assert!(reg.get("FamilySearch").is_some());
        let mut available = reg.available_providers();
        available.sort();
        assert!(available.contains(&"mock".to_string()));
        assert!(available.contains(&"familysearch".to_string()));
    }
}
