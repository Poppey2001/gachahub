use super::GameProvider;
pub struct GryphlineProvider;
impl GameProvider for GryphlineProvider {
    fn id(&self) -> &'static str { "gryphline" }
    fn supports(&self, game_id: &str) -> bool { game_id == "endfield" }
}
