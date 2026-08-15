//! HTTP REST client and API contract implementation for `mag`.

use mag_common::{AgentStatus, TaskRequest, TaskResult};
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API error response: status {status}, message: {message}")]
    Response { status: u16, message: String },
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct WorkerApiClient {
    client: Client,
}

impl Default for WorkerApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn check_health(&self, host: &str, port: u16) -> bool {
        let url = format!("http://{}:{}/health", host, port);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn send_task(&self, host: &str, port: u16, task: &TaskRequest) -> Result<(), ApiError> {
        let url = format!("http://{}:{}/task", host, port);
        let resp = self.client.post(&url).json(task).send().await?;

        if resp.status().is_success() || resp.status().as_u16() == 202 {
            Ok(())
        } else {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            Err(ApiError::Response {
                status,
                message: text,
            })
        }
    }

    pub async fn get_status(&self, host: &str, port: u16) -> Result<AgentStatus, ApiError> {
        let url = format!("http://{}:{}/status", host, port);
        let resp = self.client.get(&url).send().await?;
        let status = resp.json::<AgentStatus>().await?;
        Ok(status)
    }

    pub async fn get_result(&self, host: &str, port: u16) -> Result<Option<TaskResult>, ApiError> {
        let url = format!("http://{}:{}/result", host, port);
        let resp = self.client.get(&url).send().await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        let result = resp.json::<TaskResult>().await?;
        Ok(Some(result))
    }

    pub async fn cancel_task(&self, host: &str, port: u16) -> Result<bool, ApiError> {
        let url = format!("http://{}:{}/cancel", host, port);
        let resp = self.client.post(&url).send().await?;
        Ok(resp.status().is_success())
    }
}

/// Google OAuth2 Device Authorization Flow Response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Google OAuth2 Token Response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub id_token: Option<String>,
}

/// Google Userinfo Response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Clone)]
pub struct GoogleAuthClient {
    client: Client,
}

impl Default for GoogleAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn request_device_code(&self, client_id: &str) -> Result<DeviceCodeResponse, ApiError> {
        let url = "https://oauth2.googleapis.com/device/code";
        let params = [
            ("client_id", client_id),
            ("scope", "email profile openid"),
        ];

        let resp = self.client.post(url).form(&params).send().await?;
        if resp.status().is_success() {
            let code_resp: DeviceCodeResponse = resp.json().await?;
            Ok(code_resp)
        } else {
            let status = resp.status().as_u16();
            let msg = resp.text().await.unwrap_or_default();
            Err(ApiError::Response { status, message: msg })
        }
    }

    pub async fn poll_token(
        &self,
        client_id: &str,
        client_secret: &str,
        device_code: &str,
    ) -> Result<OAuthTokenResponse, ApiError> {
        let url = "https://oauth2.googleapis.com/token";
        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];

        let resp = self.client.post(url).form(&params).send().await?;
        if resp.status().is_success() {
            let token_resp: OAuthTokenResponse = resp.json().await?;
            Ok(token_resp)
        } else {
            let status = resp.status().as_u16();
            let msg = resp.text().await.unwrap_or_default();
            Err(ApiError::Response { status, message: msg })
        }
    }

    pub async fn fetch_user_info(&self, access_token: &str) -> Result<GoogleUserInfo, ApiError> {
        let url = "https://www.googleapis.com/oauth2/v3/userinfo";
        let resp = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?;

        if resp.status().is_success() {
            let info: GoogleUserInfo = resp.json().await?;
            Ok(info)
        } else {
            let status = resp.status().as_u16();
            let msg = resp.text().await.unwrap_or_default();
            Err(ApiError::Response { status, message: msg })
        }
    }
}
