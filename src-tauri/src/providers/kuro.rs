use super::GameProvider;
pub struct KuroProvider;
impl GameProvider for KuroProvider {
    fn id(&self) -> &'static str { "kuro" }
    fn supports(&self, game_id: &str) -> bool { matches!(game_id, "wuwa" | "pgr") }
}
