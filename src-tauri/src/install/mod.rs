use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::providers::provider_for;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallJob {
    pub id: String,
    pub game_id: String,
    pub phase: String,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes: u64,
    pub version: Option<String>,
    pub provider_mode: Option<String>,
    pub destination: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("no provider registered for {0}")]
    NoProvider(String),
}

pub fn create_install_job(game_id: &str) -> Result<InstallJob, InstallError> {
    let provider = provider_for(game_id).ok_or_else(|| InstallError::NoProvider(game_id.into()))?;
    let provider_id = provider.id();

    Ok(InstallJob {
        id: Uuid::new_v4().to_string(),
        game_id: game_id.to_string(),
        phase: "queued".into(),
        progress: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        speed_bytes: 0,
        version: None,
        provider_mode: None,
        destination: None,
        message: Some(format!("Queued for provider {provider_id}")),
    })
}
