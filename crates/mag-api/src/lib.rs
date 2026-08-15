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
