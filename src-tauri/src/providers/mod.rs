mod gryphline;
mod hoyoverse;
mod hypergryph;
mod kuro;
mod sunborn;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFile {
    pub url: String,
    pub destination: String,
    pub size: u64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlan {
    pub game_id: String,
    pub version: String,
    pub files: Vec<PackageFile>,
}

pub trait GameProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn supports(&self, game_id: &str) -> bool;
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider metadata not implemented yet for {0}")]
    NotImplemented(String),
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
