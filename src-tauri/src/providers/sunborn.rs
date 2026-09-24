use super::GameProvider;
pub struct SunbornProvider;
impl GameProvider for SunbornProvider {
    fn id(&self) -> &'static str { "sunborn" }
    fn supports(&self, game_id: &str) -> bool { game_id == "gfl2" }
}
