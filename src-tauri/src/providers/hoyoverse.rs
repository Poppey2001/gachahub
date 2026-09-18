use super::{GameProvider, InstallPlan, ProviderError};
pub struct HoyoverseProvider;
impl GameProvider for HoyoverseProvider {
    fn id(&self) -> &'static str { "hoyoverse" }
    fn supports(&self, game_id: &str) -> bool { matches!(game_id, "genshin" | "hsr" | "zzz" | "hi3") }
    fn install_plan(&self, game_id: &str) -> Result<InstallPlan, ProviderError> { Err(ProviderError::NotImplemented(game_id.into())) }
}
