use super::{GameProvider, InstallPlan, ProviderError};
pub struct HypergryphProvider;
impl GameProvider for HypergryphProvider {
    fn id(&self) -> &'static str { "hypergryph" }
    fn supports(&self, game_id: &str) -> bool { game_id == "arknights" }
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError> { Err(ProviderError::NotImplemented(game_id.into())) }
}
