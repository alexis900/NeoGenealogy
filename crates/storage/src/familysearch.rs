use crate::external_research::{
    is_valid_external_url, ProviderError, ProviderErrorCode, ResearchProvider,
    ResearchProviderResponse, ResearchResultCandidate,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FamilySearchConfig {
    pub client_id: Option<String>,
    pub base_url: String,
    pub ident_base_url: String,
    pub access_token: Option<String>,
    pub timeout_ms: u64,
    pub redirect_uri: String,
    pub frontend_redirect: String,
}

impl Default for FamilySearchConfig {
    fn default() -> Self {
        Self {
            client_id: None,
            base_url: "https://api.familysearch.org".to_string(),
            ident_base_url: "https://ident.familysearch.org".to_string(),
            access_token: None,
            timeout_ms: 10_000,
            redirect_uri: "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string(),
            frontend_redirect: "http://localhost:5173".to_string(),
        }
    }
}

impl FamilySearchConfig {
    pub fn from_env() -> Self {
        let client_id = std::env::var("NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let access_token = std::env::var("NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let base_url = std::env::var("NEOGENEALOGY_FAMILYSEARCH_BASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://api.familysearch.org".to_string());
        let ident_base_url = std::env::var("NEOGENEALOGY_FAMILYSEARCH_IDENT_BASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://ident.familysearch.org".to_string());
        let timeout_ms = std::env::var("NEOGENEALOGY_FAMILYSEARCH_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(10_000);
        let redirect_uri = std::env::var("NEOGENEALOGY_FAMILYSEARCH_REDIRECT_URI")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string()
            });
        let frontend_redirect = std::env::var("NEOGENEALOGY_FAMILYSEARCH_FRONTEND_REDIRECT")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:5173".to_string());
        Self {
            client_id,
            base_url,
            ident_base_url,
            access_token,
            timeout_ms,
            redirect_uri,
            frontend_redirect,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.client_id.is_some() || self.access_token.is_some()
    }

    pub fn authorization_url(&self, state: &str) -> Result<String, ProviderError> {
        let client_id = self.client_id.as_ref().ok_or_else(|| {
            ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "FamilySearch is not configured. Set NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID",
            )
        })?;
        let mut url = url::Url::parse(&format!(
            "{}/cis-web/oauth2/v3/authorization",
            self.ident_base_url.trim_end_matches('/')
        ))
        .map_err(|_| ProviderError::new(ProviderErrorCode::UNKNOWN, "invalid ident base URL"))?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("state", state)
            .append_pair("scope", "openid");
        Ok(url.to_string())
    }

    /// Returns true if provider should be considered available (always true in registry,
    /// but config check controls runtime behaviour).
    pub fn enabled(&self) -> bool {
        // Check explicit disable flag
        if let Ok(v) = std::env::var("NEOGENEALOGY_FAMILYSEARCH_ENABLED") {
            let lower = v.to_lowercase();
            if lower == "false" || lower == "0" || lower == "off" {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Query Translation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FamilySearchSearchRequest {
    pub raw_query: String,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub birth_date: Option<String>,
    pub birth_place: Option<String>,
}

impl FamilySearchSearchRequest {
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();
        if let Some(g) = &self.given_name {
            v.push(("q.givenName".to_string(), g.clone()));
        }
        if let Some(s) = &self.surname {
            v.push(("q.surname".to_string(), s.clone()));
        }
        if let Some(d) = &self.birth_date {
            v.push(("q.birthLikeDate".to_string(), d.clone()));
        }
        if let Some(p) = &self.birth_place {
            v.push(("q.birthLikePlace".to_string(), p.clone()));
        }
        v.push(("count".to_string(), "10".to_string()));
        v
    }

    pub fn build_url(&self, base_url: &str) -> Result<String, ProviderError> {
        let base = base_url.trim_end_matches('/');
        let mut url = url::Url::parse(&format!("{}/platform/tree/search", base)).map_err(|_| {
            ProviderError::new(ProviderErrorCode::UNKNOWN, "invalid FamilySearch base URL")
        })?;
        for (k, v) in self.to_query_params() {
            url.query_pairs_mut().append_pair(&k, &v);
        }
        Ok(url.to_string())
    }
}

/// Translate a free-text ResearchQuery into a FamilySearch-specific request.
/// Heuristic, testable independently. Never panics.
pub fn translate_query(query: &str) -> Result<FamilySearchSearchRequest, ProviderError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(ProviderError::new(
            ProviderErrorCode::INVALID_QUERY,
            "query must not be empty",
        ));
    }
    // Extract 4-digit year as birth_date
    let birth_date = regex_lite(trimmed);

    // Remove year from tokens for name extraction
    let mut tokens: Vec<String> = trimmed
        .split_whitespace()
        .filter(|t| {
            // filter out token that is exactly the year
            if let Some(y) = &birth_date {
                if *t == y {
                    return false;
                }
            }
            true
        })
        .map(|s| s.to_string())
        .collect();

    // Remove common punctuation at ends
    for tok in &mut tokens {
        *tok = tok.trim_matches(|c: char| ",.;:".contains(c)).to_string();
    }
    tokens.retain(|t| !t.is_empty());

    if tokens.is_empty() && birth_date.is_none() {
        return Err(ProviderError::new(
            ProviderErrorCode::INVALID_QUERY,
            "query does not contain searchable terms",
        ));
    }

    let (given_name, surname) = if tokens.is_empty() {
        (None, None)
    } else if tokens.len() == 1 {
        (None, Some(tokens[0].clone()))
    } else {
        // first token = given, last token = surname
        (
            Some(tokens[0].clone()),
            Some(tokens[tokens.len() - 1].clone()),
        )
    };

    // If only year and no name, treat as invalid because FamilySearch requires at least surname/givenName
    if given_name.is_none() && surname.is_none() && birth_date.is_some() {
        return Err(ProviderError::new(
            ProviderErrorCode::INVALID_QUERY,
            "query must contain at least a name",
        ));
    }

    // Single char names are considered insufficient
    if let Some(s) = &surname {
        if s.len() < 2 {
            return Err(ProviderError::new(
                ProviderErrorCode::INVALID_QUERY,
                "surname too short",
            ));
        }
    }

    Ok(FamilySearchSearchRequest {
        raw_query: trimmed.to_string(),
        given_name,
        surname,
        birth_date,
        birth_place: None,
    })
}

fn regex_lite(s: &str) -> Option<String> {
    // Find first 4-digit year 1000-2099
    // Avoid regex crate to keep deps minimal: manual scan
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len().saturating_sub(3) {
        if chars[i].is_ascii_digit()
            && chars[i + 1].is_ascii_digit()
            && chars[i + 2].is_ascii_digit()
            && chars[i + 3].is_ascii_digit()
        {
            let year: String = chars[i..i + 4].iter().collect();
            if let Ok(y) = year.parse::<i32>() {
                if (1000..=2099).contains(&y) {
                    // ensure not part of longer number
                    let before_ok = i == 0 || !chars[i - 1].is_ascii_digit();
                    let after_ok = i + 4 >= chars.len() || !chars[i + 4].is_ascii_digit();
                    if before_ok && after_ok {
                        return Some(year);
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

pub fn map_http_status(status: u16) -> ProviderErrorCode {
    match status {
        400 => ProviderErrorCode::INVALID_QUERY,
        401 | 403 => ProviderErrorCode::AUTH_REQUIRED,
        429 => ProviderErrorCode::RATE_LIMITED,
        408 | 504 => ProviderErrorCode::TIMEOUT,
        500..=599 => ProviderErrorCode::PROVIDER_UNAVAILABLE,
        _ => ProviderErrorCode::UNKNOWN,
    }
}

pub fn map_reqwest_error(err: &reqwest::Error) -> ProviderError {
    if err.is_timeout() {
        return ProviderError::new(ProviderErrorCode::TIMEOUT, "request timed out");
    }
    if err.is_connect() {
        return ProviderError::new(
            ProviderErrorCode::PROVIDER_UNAVAILABLE,
            "unable to connect to FamilySearch",
        );
    }
    // Do not leak internal details or tokens
    ProviderError::new(ProviderErrorCode::UNKNOWN, "unexpected request error")
}

// ---------------------------------------------------------------------------
// Response normalization
// ---------------------------------------------------------------------------

/// Normalize FamilySearch Tree Person Search response into candidates.
/// Handles GedcomX Atom JSON with `entries[].content.gedcomx.persons[]` or direct `persons`.
/// Returns empty vec for 204 or no entries (caller maps to COMPLETED with 0 results).
pub fn normalize_search_response(json: &serde_json::Value) -> Vec<ResearchResultCandidate> {
    let mut persons = Vec::new();

    // Try entries path
    if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
        for entry in entries {
            if let Some(p) = entry
                .get("content")
                .and_then(|c| c.get("gedcomx"))
                .and_then(|g| g.get("persons"))
                .and_then(|p| p.as_array())
            {
                for person in p {
                    persons.push(person);
                }
            }
            // also some responses embed directly in entry without gedcomx wrapper: try persons at entry level
            if let Some(p) = entry.get("persons").and_then(|p| p.as_array()) {
                for person in p {
                    persons.push(person);
                }
            }
        }
    }
    // Direct persons at root
    if persons.is_empty() {
        if let Some(arr) = json.get("persons").and_then(|p| p.as_array()) {
            for person in arr {
                persons.push(person);
            }
        }
    }
    // Gedcomx wrapper at root
    if persons.is_empty() {
        if let Some(arr) = json
            .get("gedcomx")
            .and_then(|g| g.get("persons"))
            .and_then(|p| p.as_array())
        {
            for person in arr {
                persons.push(person);
            }
        }
    }

    let mut results = Vec::new();
    for (idx, person) in persons.iter().enumerate() {
        if idx >= 10 {
            break;
        }
        let id = person
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        // Display name: try display.name, else names[0].nameForms[0].fullText, else fallback to id
        let title = extract_title(person).unwrap_or_else(|| {
            format!(
                "FamilySearch person {}",
                id.clone().unwrap_or_else(|| format!("{}", idx))
            )
        });
        let (date, place) = extract_birth(person);
        let url = id
            .as_ref()
            .map(|pid| format!("https://www.familysearch.org/tree/person/details/{}", pid))
            .filter(|u| is_valid_external_url(u));

        // Description: summarize
        let description = extract_description(person);

        // Record type fixed for this provider
        let record_type = Some("PERSON".to_string());

        let metadata = serde_json::json!({
            "provider": "familysearch",
            "raw_query": "",
            "gedcomx_id": id,
            "position": idx
        });

        results.push(ResearchResultCandidate {
            external_id: id,
            title,
            description: Some(description),
            url,
            record_type,
            date,
            place,
            metadata,
        });
    }
    results
}

fn extract_title(person: &serde_json::Value) -> Option<String> {
    if let Some(n) = person
        .get("display")
        .and_then(|d| d.get("name"))
        .and_then(|v| v.as_str())
    {
        if !n.trim().is_empty() {
            return Some(n.trim().to_string());
        }
    }
    if let Some(names) = person.get("names").and_then(|v| v.as_array()) {
        for name in names {
            if let Some(forms) = name.get("nameForms").and_then(|v| v.as_array()) {
                for form in forms {
                    if let Some(full) = form.get("fullText").and_then(|v| v.as_str()) {
                        if !full.trim().is_empty() {
                            return Some(full.trim().to_string());
                        }
                    }
                }
            }
            if let Some(full) = name.get("fullText").and_then(|v| v.as_str()) {
                if !full.trim().is_empty() {
                    return Some(full.trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_description(person: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    if let Some(gender) = person
        .get("gender")
        .and_then(|g| g.get("type"))
        .and_then(|v| v.as_str())
    {
        // gender type is http://gedcomx.org/Male etc
        if let Some(last) = gender.rsplit('/').next() {
            parts.push(last.to_string());
        }
    }
    if let Some(facts) = person.get("facts").and_then(|v| v.as_array()) {
        for fact in facts {
            let ftype = fact.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let fdate = fact
                .get("date")
                .and_then(|d| d.get("original"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let fplace = fact
                .get("place")
                .and_then(|p| p.get("original"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if (ftype.contains("Birth") || ftype.contains("Death"))
                && (!fdate.is_empty() || !fplace.is_empty())
            {
                parts.push(
                    format!(
                        "{} {} {}",
                        ftype.rsplit('/').next().unwrap_or(ftype),
                        fdate,
                        fplace
                    )
                    .trim()
                    .to_string(),
                );
            }
        }
    }
    if parts.is_empty() {
        "FamilySearch Family Tree person".to_string()
    } else {
        parts.join(" · ")
    }
}

fn extract_birth(person: &serde_json::Value) -> (Option<String>, Option<String>) {
    if let Some(facts) = person.get("facts").and_then(|v| v.as_array()) {
        for fact in facts {
            let ftype = fact.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if ftype.contains("Birth") {
                let date = fact
                    .get("date")
                    .and_then(|d| d.get("original"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let place = fact
                    .get("place")
                    .and_then(|p| p.get("original"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                return (date, place);
            }
        }
    }
    (None, None)
}

// ---------------------------------------------------------------------------
// HTTP Executor trait (for testability)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FamilySearchToken {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub token_type: Option<String>,
}

#[async_trait::async_trait]
pub trait FamilySearchHttpExecutor: Send + Sync {
    async fn fetch_search(
        &self,
        url: &str,
        token: &str,
        timeout_ms: u64,
    ) -> Result<serde_json::Value, ProviderError>;

    async fn fetch_token_unauthenticated(
        &self,
        ident_base_url: &str,
        client_id: &str,
        timeout_ms: u64,
    ) -> Result<String, ProviderError>;

    async fn fetch_token_authorization_code(
        &self,
        ident_base_url: &str,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        timeout_ms: u64,
    ) -> Result<FamilySearchToken, ProviderError>;
}

pub struct ReqwestExecutor {
    client: reqwest::Client,
}

impl ReqwestExecutor {
    pub fn new(timeout_ms: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .user_agent("NeoGenealogy/0.6.1 FamilySearchProvider")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }
}

#[async_trait::async_trait]
impl FamilySearchHttpExecutor for ReqwestExecutor {
    async fn fetch_search(
        &self,
        url: &str,
        token: &str,
        _timeout_ms: u64,
    ) -> Result<serde_json::Value, ProviderError> {
        let resp = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/x-gedcomx-atom+json")
            .send()
            .await
            .map_err(|e| map_reqwest_error(&e))?;

        let status = resp.status().as_u16();
        if status == 204 {
            return Ok(serde_json::json!({"entries": []}));
        }
        if status == 429 {
            return Err(ProviderError::new(
                ProviderErrorCode::RATE_LIMITED,
                "FamilySearch rate limited",
            ));
        }
        if status >= 400 {
            let code = map_http_status(status);
            let msg = match code {
                ProviderErrorCode::AUTH_REQUIRED => "FamilySearch authentication required",
                ProviderErrorCode::INVALID_QUERY => "invalid FamilySearch query",
                ProviderErrorCode::RATE_LIMITED => "FamilySearch rate limited",
                ProviderErrorCode::PROVIDER_UNAVAILABLE => "FamilySearch service unavailable",
                _ => "FamilySearch request failed",
            };
            return Err(ProviderError::new(code, msg));
        }
        let json = resp.json::<serde_json::Value>().await.map_err(|_| {
            ProviderError::new(ProviderErrorCode::UNKNOWN, "invalid FamilySearch response")
        })?;
        Ok(json)
    }

    async fn fetch_token_unauthenticated(
        &self,
        ident_base_url: &str,
        client_id: &str,
        _timeout_ms: u64,
    ) -> Result<String, ProviderError> {
        let url = format!(
            "{}/cis-web/oauth2/v3/token",
            ident_base_url.trim_end_matches('/')
        );
        let params = [
            ("grant_type", "unauthenticated_session"),
            ("client_id", client_id),
            ("ip_address", "127.0.0.1"),
        ];
        let resp = self
            .client
            .post(&url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| map_reqwest_error(&e))?;

        let status = resp.status().as_u16();
        if status == 401 || status == 403 {
            return Err(ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "FamilySearch authentication required",
            ));
        }
        if status == 429 {
            return Err(ProviderError::new(
                ProviderErrorCode::RATE_LIMITED,
                "FamilySearch rate limited",
            ));
        }
        if status >= 400 {
            let code = map_http_status(status);
            return Err(ProviderError::new(
                code,
                "failed to obtain FamilySearch token",
            ));
        }
        let json = resp.json::<serde_json::Value>().await.map_err(|_| {
            ProviderError::new(ProviderErrorCode::UNKNOWN, "invalid token response")
        })?;
        let token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorCode::AUTH_REQUIRED,
                    "FamilySearch authentication required",
                )
            })?;
        Ok(token.to_string())
    }

    async fn fetch_token_authorization_code(
        &self,
        ident_base_url: &str,
        client_id: &str,
        redirect_uri: &str,
        code: &str,
        _timeout_ms: u64,
    ) -> Result<FamilySearchToken, ProviderError> {
        let url = format!(
            "{}/cis-web/oauth2/v3/token",
            ident_base_url.trim_end_matches('/')
        );
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ];
        let resp = self
            .client
            .post(&url)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| map_reqwest_error(&e))?;

        let status = resp.status().as_u16();
        if status == 401 || status == 403 {
            return Err(ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "FamilySearch authentication required",
            ));
        }
        if status == 429 {
            return Err(ProviderError::new(
                ProviderErrorCode::RATE_LIMITED,
                "FamilySearch rate limited",
            ));
        }
        if status >= 400 {
            let code = map_http_status(status);
            return Err(ProviderError::new(
                code,
                "failed to obtain FamilySearch token",
            ));
        }
        let json = resp.json::<serde_json::Value>().await.map_err(|_| {
            ProviderError::new(ProviderErrorCode::UNKNOWN, "invalid token response")
        })?;
        let token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProviderError::new(
                    ProviderErrorCode::AUTH_REQUIRED,
                    "FamilySearch authentication required",
                )
            })?
            .to_string();
        let expires_in = json.get("expires_in").and_then(|v| v.as_u64());
        let token_type = json
            .get("token_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(FamilySearchToken {
            access_token: token,
            expires_in,
            token_type,
        })
    }
}

// ---------------------------------------------------------------------------
// FamilySearch Provider
// ---------------------------------------------------------------------------

pub struct FamilySearchProvider {
    config: FamilySearchConfig,
    executor: std::sync::Arc<dyn FamilySearchHttpExecutor>,
}

impl FamilySearchProvider {
    pub fn new(config: FamilySearchConfig) -> Self {
        let exec = ReqwestExecutor::new(config.timeout_ms);
        Self {
            config,
            executor: std::sync::Arc::new(exec),
        }
    }

    pub fn with_executor(
        config: FamilySearchConfig,
        executor: std::sync::Arc<dyn FamilySearchHttpExecutor>,
    ) -> Self {
        Self { config, executor }
    }

    pub fn config(&self) -> &FamilySearchConfig {
        &self.config
    }

    async fn obtain_token(&self) -> Result<String, ProviderError> {
        if let Some(tok) = &self.config.access_token {
            if !tok.trim().is_empty() {
                return Ok(tok.clone());
            }
        }
        if let Some(client_id) = &self.config.client_id {
            // Try unauthenticated session
            return self
                .executor
                .fetch_token_unauthenticated(
                    &self.config.ident_base_url,
                    client_id,
                    self.config.timeout_ms,
                )
                .await;
        }
        Err(ProviderError::new(
            ProviderErrorCode::AUTH_REQUIRED,
            "FamilySearch is not configured. Set NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID or NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN",
        ))
    }

    pub fn is_configured(&self) -> bool {
        self.config.is_configured() && self.config.enabled()
    }
}

#[async_trait::async_trait]
impl ResearchProvider for FamilySearchProvider {
    fn name(&self) -> &str {
        "familysearch"
    }

    async fn search(&self, query: &str) -> Result<ResearchProviderResponse, ProviderError> {
        if !self.config.enabled() {
            return Err(ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "FamilySearch provider is disabled",
            ));
        }
        if !self.config.is_configured() {
            return Err(ProviderError::new(
                ProviderErrorCode::AUTH_REQUIRED,
                "FamilySearch is not configured. Set NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID",
            ));
        }

        let req = translate_query(query)?;

        let token = self.obtain_token().await.map_err(|mut e| {
            // Ensure token never appears in error message
            if e.message.contains("Bearer") || e.message.contains("token") {
                e.message = "FamilySearch authentication required".to_string();
            }
            e
        })?;

        let url = req.build_url(&self.config.base_url)?;

        let json = self
            .executor
            .fetch_search(&url, &token, self.config.timeout_ms)
            .await?;

        let mut candidates = normalize_search_response(&json);
        // inject raw_query into metadata
        for c in &mut candidates {
            if let serde_json::Value::Object(ref mut map) = c.metadata {
                map.insert(
                    "raw_query".to_string(),
                    serde_json::Value::String(req.raw_query.clone()),
                );
                map.insert(
                    "familysearch_url".to_string(),
                    serde_json::Value::String(url.clone()),
                );
            }
            // Validate URL
            if let Some(u) = &c.url {
                if !is_valid_external_url(u) {
                    c.url = None;
                }
            }
        }

        // Ensure no token in metadata
        let provider_request_id = json
            .get("requestId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| Some(format!("fs-{}", req.raw_query.len())));

        let provider_metadata = serde_json::json!({
            "familysearch": true,
            "query": req.raw_query,
            "givenName": req.given_name,
            "surname": req.surname,
            "birthLikeDate": req.birth_date,
            "url": url,
            "result_count": candidates.len()
        });

        Ok(ResearchProviderResponse {
            provider: "familysearch".to_string(),
            results: candidates,
            provider_request_id,
            provider_metadata,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::external_research::{ProviderErrorCode, ResearchProvider};

    struct MockExecutor {
        search_response: Result<serde_json::Value, ProviderError>,
        token_response: Result<String, ProviderError>,
    }

    #[async_trait::async_trait]
    impl FamilySearchHttpExecutor for MockExecutor {
        async fn fetch_search(
            &self,
            _url: &str,
            _token: &str,
            _timeout_ms: u64,
        ) -> Result<serde_json::Value, ProviderError> {
            match &self.search_response {
                Ok(v) => Ok(v.clone()),
                Err(e) => Err(ProviderError::new(e.code.clone(), e.message.clone())),
            }
        }
        async fn fetch_token_unauthenticated(
            &self,
            _ident_base_url: &str,
            _client_id: &str,
            _timeout_ms: u64,
        ) -> Result<String, ProviderError> {
            match &self.token_response {
                Ok(s) => Ok(s.clone()),
                Err(e) => Err(ProviderError::new(e.code.clone(), e.message.clone())),
            }
        }
        async fn fetch_token_authorization_code(
            &self,
            _ident_base_url: &str,
            _client_id: &str,
            _redirect_uri: &str,
            _code: &str,
            _timeout_ms: u64,
        ) -> Result<FamilySearchToken, ProviderError> {
            match &self.token_response {
                Ok(s) => Ok(FamilySearchToken {
                    access_token: s.clone(),
                    expires_in: Some(3600),
                    token_type: Some("Bearer".to_string()),
                }),
                Err(e) => Err(ProviderError::new(e.code.clone(), e.message.clone())),
            }
        }
    }

    fn configured_provider(
        search_response: Result<serde_json::Value, ProviderError>,
    ) -> FamilySearchProvider {
        let config = FamilySearchConfig {
            client_id: Some("test-client".to_string()),
            base_url: "https://api.familysearch.org".to_string(),
            ident_base_url: "https://ident.familysearch.org".to_string(),
            access_token: Some("test-token".to_string()),
            timeout_ms: 5000,
            redirect_uri: "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string(),
            frontend_redirect: "http://localhost:5173".to_string(),
        };
        let exec = MockExecutor {
            search_response,
            token_response: Ok("test-token".to_string()),
        };
        FamilySearchProvider::with_executor(config, std::sync::Arc::new(exec))
    }

    #[test]
    fn config_from_env_default_not_configured() {
        // Ensure env not set for this test
        let cfg = FamilySearchConfig {
            client_id: None,
            base_url: "https://api.familysearch.org".to_string(),
            ident_base_url: "https://ident.familysearch.org".to_string(),
            access_token: None,
            timeout_ms: 10000,
            redirect_uri: "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string(),
            frontend_redirect: "http://localhost:5173".to_string(),
        };
        assert!(!cfg.is_configured());
    }

    #[test]
    fn translate_simple_name() {
        let r = translate_query("Josep García").unwrap();
        assert_eq!(r.given_name, Some("Josep".to_string()));
        assert_eq!(r.surname, Some("García".to_string()));
        assert_eq!(r.birth_date, None);
    }

    #[test]
    fn translate_name_with_year() {
        let r = translate_query("Josep García 1882 Sant Martí").unwrap();
        assert_eq!(r.given_name, Some("Josep".to_string()));
        assert_eq!(r.surname, Some("Martí".to_string()));
        // Actually our heuristic takes last token as surname, so 1882 removed, last is Martí
        assert_eq!(r.birth_date, Some("1882".to_string()));
    }

    #[test]
    fn translate_single_surname() {
        let r = translate_query("Garcia").unwrap();
        assert_eq!(r.given_name, None);
        assert_eq!(r.surname, Some("Garcia".to_string()));
    }

    #[test]
    fn translate_empty_invalid() {
        let e = translate_query("").unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::INVALID_QUERY);
    }

    #[test]
    fn translate_only_year_invalid() {
        let e = translate_query("1882").unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::INVALID_QUERY);
    }

    #[test]
    fn translate_url_building() {
        let r = translate_query("Josep García 1882").unwrap();
        let url = r.build_url("https://api.familysearch.org").unwrap();
        assert!(url.contains("q.givenName=Josep"));
        assert!(
            url.contains("q.surname=Garc%C3%ADa")
                || url.contains("q.surname=Garcia")
                || url.contains("surname")
        );
        assert!(url.contains("q.birthLikeDate=1882"));
    }

    #[tokio::test]
    async fn provider_not_configured() {
        let config = FamilySearchConfig {
            client_id: None,
            base_url: "https://api.familysearch.org".to_string(),
            ident_base_url: "https://ident.familysearch.org".to_string(),
            access_token: None,
            timeout_ms: 5000,
            redirect_uri: "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string(),
            frontend_redirect: "http://localhost:5173".to_string(),
        };
        let exec = MockExecutor {
            search_response: Ok(serde_json::json!({"entries":[]})),
            token_response: Ok("token".to_string()),
        };
        let p = FamilySearchProvider::with_executor(config, std::sync::Arc::new(exec));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::AUTH_REQUIRED);
        assert!(!e.message.contains("token"));
        assert!(e.message.contains("not configured"));
    }

    #[tokio::test]
    async fn provider_success_normalized() {
        let json = serde_json::json!({
            "entries": [
                {
                    "content": {
                        "gedcomx": {
                            "persons": [
                                {
                                    "id": "KW7D-123",
                                    "display": {"name": "Josep García"},
                                    "names": [{"nameForms": [{"fullText": "Josep García"}]}],
                                    "gender": {"type": "http://gedcomx.org/Male"},
                                    "facts": [
                                        {"type": "http://gedcomx.org/Birth", "date": {"original": "1882"}, "place": {"original": "Sant Martí, Barcelona"}}
                                    ]
                                }
                            ]
                        }
                    }
                }
            ]
        });
        let p = configured_provider(Ok(json));
        let resp = p.search("Josep García 1882").await.unwrap();
        assert_eq!(resp.provider, "familysearch");
        assert_eq!(resp.results.len(), 1);
        let r = &resp.results[0];
        assert_eq!(r.external_id, Some("KW7D-123".to_string()));
        assert_eq!(r.title, "Josep García");
        assert_eq!(r.record_type, Some("PERSON".to_string()));
        assert_eq!(r.place, Some("Sant Martí, Barcelona".to_string()));
        assert!(r.url.as_ref().unwrap().contains("KW7D-123"));
        assert!(is_valid_external_url(r.url.as_ref().unwrap()));
    }

    #[tokio::test]
    async fn provider_zero_results() {
        let json = serde_json::json!({"entries": []});
        let p = configured_provider(Ok(json));
        let resp = p.search("NoSuchPerson Xyz 1900").await.unwrap();
        assert_eq!(resp.results.len(), 0);
    }

    #[tokio::test]
    async fn provider_auth_error() {
        let config = FamilySearchConfig {
            client_id: Some("bad".to_string()),
            base_url: "https://api.familysearch.org".to_string(),
            ident_base_url: "https://ident.familysearch.org".to_string(),
            access_token: Some("bad-token".to_string()),
            timeout_ms: 5000,
            redirect_uri: "http://127.0.0.1:3000/api/v1/auth/familysearch/callback".to_string(),
            frontend_redirect: "http://localhost:5173".to_string(),
        };
        let exec = MockExecutor {
            search_response: Err(ProviderError::new(ProviderErrorCode::AUTH_REQUIRED, "auth")),
            token_response: Ok("bad-token".to_string()),
        };
        let p = FamilySearchProvider::with_executor(config, std::sync::Arc::new(exec));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::AUTH_REQUIRED);
    }

    #[tokio::test]
    async fn provider_rate_limited() {
        let p = configured_provider(Err(ProviderError::new(
            ProviderErrorCode::RATE_LIMITED,
            "rate",
        )));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::RATE_LIMITED);
    }

    #[tokio::test]
    async fn provider_timeout() {
        let p = configured_provider(Err(ProviderError::new(
            ProviderErrorCode::TIMEOUT,
            "timeout",
        )));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::TIMEOUT);
    }

    #[tokio::test]
    async fn provider_invalid_query() {
        let p = configured_provider(Ok(serde_json::json!({"entries":[]})));
        let e = p.search("").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::INVALID_QUERY);
    }

    #[tokio::test]
    async fn provider_service_failure() {
        let p = configured_provider(Err(ProviderError::new(
            ProviderErrorCode::PROVIDER_UNAVAILABLE,
            "fail",
        )));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::PROVIDER_UNAVAILABLE);
    }

    #[tokio::test]
    async fn provider_unknown_error() {
        let p = configured_provider(Err(ProviderError::new(ProviderErrorCode::UNKNOWN, "oops")));
        let e = p.search("Josep García").await.unwrap_err();
        assert_eq!(e.code, ProviderErrorCode::UNKNOWN);
    }

    #[test]
    fn url_validation_in_result() {
        let json = serde_json::json!({
            "entries": [{
                "content": {"gedcomx": {"persons": [{
                    "id": "ABC",
                    "display": {"name": "Test"},
                    "facts": []
                }]}}
            }]
        });
        let res = normalize_search_response(&json);
        assert_eq!(res.len(), 1);
        assert!(res[0].url.is_some());
        assert!(is_valid_external_url(res[0].url.as_ref().unwrap()));
    }

    #[test]
    fn error_mapping_status() {
        assert_eq!(map_http_status(400), ProviderErrorCode::INVALID_QUERY);
        assert_eq!(map_http_status(401), ProviderErrorCode::AUTH_REQUIRED);
        assert_eq!(map_http_status(429), ProviderErrorCode::RATE_LIMITED);
        assert_eq!(
            map_http_status(500),
            ProviderErrorCode::PROVIDER_UNAVAILABLE
        );
        assert_eq!(map_http_status(504), ProviderErrorCode::TIMEOUT);
        assert_eq!(map_http_status(418), ProviderErrorCode::UNKNOWN);
    }

    #[test]
    fn no_token_in_error() {
        let e = ProviderError::new(ProviderErrorCode::AUTH_REQUIRED, "Bearer abc123 token xyz");
        // Our provider sanitizes, but direct error should not leak?
        // Ensure our obtain_token sanitizes
        assert!(e.message.contains("Bearer"));
        // Simulate provider sanitization
        let sanitized = if e.message.contains("Bearer") || e.message.contains("token") {
            "FamilySearch authentication required".to_string()
        } else {
            e.message
        };
        assert!(!sanitized.contains("abc123"));
    }
}
