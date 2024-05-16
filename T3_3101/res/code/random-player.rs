pub struct RandomPlayer {
    name: String,
    rng: Xoshiro256PlusPlus,
}
impl RandomPlayer {
    pub fn new(name: impl Into<String>, options: Option<RandomOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            name: name.into(),
            rng: Xoshiro256PlusPlus::seed_from_u64( options.seed ),
        }
    }
}
impl Player for RandomPlayer {
    fn name(&self) -> &str {
        &self.name
    }
    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        game.get_valid_actions()
            .choose(&mut self.rng)
            .copied()
            .ok_or_else(|| anyhow!("No valid actions"))
    }
}
