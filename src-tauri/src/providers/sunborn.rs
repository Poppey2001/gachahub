use super::{GameProvider, InstallPlan, ProviderError};
pub struct SunbornProvider;
impl GameProvider for SunbornProvider {
    fn id(&self) -> &'static str { "sunborn" }
    fn supports(&self, game_id: &str) -> bool { game_id == "gfl2" }
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError> { Err(ProviderError::NotImplemented(game_id.into())) }
}
