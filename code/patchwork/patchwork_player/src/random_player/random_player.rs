use patchwork_core::{Action, Game, Patchwork, Player};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

pub struct RandomPlayer {
    pub name: String,
    pub seed: Option<u64>,
    rng: Xoshiro256PlusPlus,
}

impl RandomPlayer {
    /// Creates a new [`RandomPlayer`].
    pub fn new(name: String, seed: Option<u64>) -> Self {
        let rng = Xoshiro256PlusPlus::seed_from_u64(seed.unwrap_or_else(rand::random));

        RandomPlayer { name, seed, rng }
    }
}

impl Player for RandomPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> Action {
        let mut valid_actions = game.get_valid_actions();
        let random_index = self.rng.gen_range(0..valid_actions.len());
        valid_actions.remove(random_index)
    }
}
