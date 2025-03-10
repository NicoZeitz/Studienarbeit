use crate::RandomOptions;
use patchwork_core::{ActionId, Patchwork, Player, PlayerResult};
use rand::{seq::SliceRandom, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use anyhow::anyhow;

/// A computer player that takes random actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomPlayer {
    /// The name of the player.
    name: String,
    /// The random number generator used to choose actions.
    rng: Xoshiro256PlusPlus,
}

impl RandomPlayer {
    /// Creates a new [`RandomPlayer`] with the given name and options.
    pub fn new(name: impl Into<String>, options: Option<RandomOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            name: name.into(),
            rng: Xoshiro256PlusPlus::seed_from_u64(options.seed),
        }
    }
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self::new("Random Player".to_string(), None)
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

        // let mut valid_actions = game.get_valid_actions().into_iter().collect::<Vec<_>>();
        // let random_index = self.rng.gen_range(0..valid_actions.len());
        // Ok(valid_actions.remove(random_index))
    }
}
