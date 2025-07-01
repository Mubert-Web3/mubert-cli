use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

#[derive(Deserialize, Debug)]
pub struct StatusResponse {
    pub id: String,
    pub status: String,
    pub url: String,
}

pub async fn check_fingerprint_status(
    task_id: &String,
    auth_token: &String,
) -> Result<StatusResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://fingerprint.mubert.xyz/v1/fingerprint/status?task_id={task_id}");

    let response = client
        .get(&url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {auth_token}"),
        )
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    Ok(response.json().await?)
}

pub async fn wait_for_fingerprint_url(
    task_id: &String,
    timeout_secs: u64,
    auth_token: &String,
) -> Result<String, Box<dyn Error>> {
    Retry::spawn(FixedInterval::from_millis(timeout_secs * 1000), || async {
        match check_fingerprint_status(task_id, auth_token).await {
            Ok(result) => {
                if result.status == "done" {
                    Ok(result.url)
                } else {
                    Err("fingerprint not ready".into())
                }
            }
            Err(e) => Err(e),
        }
    })
    .await
}

#[derive(Deserialize, Debug)]
pub struct JobResponse {
    pub id: String,
}

pub async fn upload_audio(
    file_path: &std::path::PathBuf,
    auth_token: &String,
) -> Result<JobResponse, Box<dyn Error>> {
    let buffer = fs::read(file_path)?;

    let client = reqwest::Client::new();
    let response = client
        .put("https://fingerprint.mubert.xyz/v1/fingerprint/create")
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {auth_token}"),
        )
        .body(buffer)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    Ok(response.json().await?)
}

#[derive(Serialize)]
pub struct MetadataRequest {
    pub title: String,
    pub bpm: u32,
    pub key: u8,
    pub scale: u8,
    pub instrument: u8,
    pub fingerprint: String,
}

#[derive(Deserialize, Debug)]
pub struct MetadataResponse {
    pub url: String,
}

pub async fn create_metadata(
    payload: MetadataRequest,
    auth_token: &String,
) -> Result<MetadataResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://fingerprint.mubert.xyz/v1/metadata/create")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {auth_token}"),
        )
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!(
            "Request failed with status: {} {}",
            response.status(),
            response.text().await.unwrap()
        )
        .into());
    }

    Ok(response.json().await?)
}
