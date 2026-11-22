mod gryphline;
pub mod hoyoverse;
mod hypergryph;
mod kuro;
mod sunborn;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageFile {
    pub url: String,
    pub destination: String,
    pub size: u64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDownloadPlan {
    pub game_id: String,
    pub provider: String,
    pub version: String,
    pub mode: String,
    pub packages: Vec<PackageFile>,
    pub total_bytes: u64,
    pub resource_list_url: Option<String>,
    pub notes: Vec<String>,
}

pub trait GameProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn supports(&self, game_id: &str) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network request failed: {0}")]
    Network(String),
    #[error("provider response is invalid: {0}")]
    InvalidResponse(String),
    #[error("game not found in provider metadata: {0}")]
    GameNotFound(String),
}

pub fn provider_for(game_id: &str) -> Option<Box<dyn GameProvider>> {
    let providers: Vec<Box<dyn GameProvider>> = vec![
        Box::new(hoyoverse::HoyoverseProvider),
        Box::new(hypergryph::HypergryphProvider),
        Box::new(gryphline::GryphlineProvider),
        Box::new(kuro::KuroProvider),
        Box::new(sunborn::SunbornProvider),
    ];
    providers.into_iter().find(|p| p.supports(game_id))
}
