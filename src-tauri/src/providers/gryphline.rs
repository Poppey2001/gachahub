use super::{GameProvider, InstallPlan, ProviderError};
pub struct GryphlineProvider;
impl GameProvider for GryphlineProvider {
    fn id(&self) -> &'static str { "gryphline" }
    fn supports(&self, game_id: &str) -> bool { game_id == "endfield" }
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError> { Err(ProviderError::NotImplemented(game_id.into())) }
}
