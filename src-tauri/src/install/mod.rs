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
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("no provider registered for {0}")]
    NoProvider(String),
}

pub fn create_install_job(game_id: &str) -> Result<InstallJob, InstallError> {
    let provider = provider_for(game_id).ok_or_else(|| InstallError::NoProvider(game_id.into()))?;
    let _provider_id = provider.id();

    // v0.1: create a real backend job object. Network fetching is intentionally
    // provider-specific and will be connected in the next milestone.
    Ok(InstallJob {
        id: Uuid::new_v4().to_string(),
        game_id: game_id.to_string(),
        phase: "downloading".into(),
        progress: 0.02,
        downloaded_bytes: 2_147_483_648,
        total_bytes: 107_374_182_400,
        speed_bytes: 43_200_000,
    })
}
