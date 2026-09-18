use super::{GameProvider, InstallPlan, ProviderError};
pub struct KuroProvider;
impl GameProvider for KuroProvider {
    fn id(&self) -> &'static str { "kuro" }
    fn supports(&self, game_id: &str) -> bool { matches!(game_id, "wuwa" | "pgr") }
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError> { Err(ProviderError::NotImplemented(game_id.into())) }
}
