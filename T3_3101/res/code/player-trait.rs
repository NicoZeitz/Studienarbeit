pub trait Player {
    fn name(&self) -> &str;

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId>;
}
