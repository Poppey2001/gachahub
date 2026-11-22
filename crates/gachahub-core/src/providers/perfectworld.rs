use super::GameProvider;
pub struct PerfectWorldProvider;
impl GameProvider for PerfectWorldProvider {
    fn id(&self) -> &'static str {
        "perfectworld"
    }
    fn supports(&self, game_id: &str) -> bool {
        matches!(game_id, "nte")
    }
}
