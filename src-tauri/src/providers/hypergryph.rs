use super::GameProvider;
pub struct HypergryphProvider;
impl GameProvider for HypergryphProvider {
    fn id(&self) -> &'static str { "hypergryph" }
    fn supports(&self, game_id: &str) -> bool { game_id == "arknights" }
}
