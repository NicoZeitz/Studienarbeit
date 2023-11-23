use std::fs;
use std::num::NonZeroUsize;

use patchwork_core::{Action, Patchwork, Player};

use crate::{mcts_player::search_tree::SearchTree, MCTSOptions, ScoreEvaluator};

use crate::{MCTSSpecification, ScoredUCTPolicy};

/// A player that uses Monte Carlo Tree Search to select actions.
pub struct MCTSPlayer {
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The name of the player.
    pub name: String,
    /// The policy to select nodes during the selection phase.
    pub policy: ScoredUCTPolicy,
    /// The evaluator to evaluate the game state.
    pub evaluator: ScoreEvaluator,
}

impl MCTSPlayer {
    /// Creates a new [`MCTSPlayer`].
    pub fn new(name: String, options: Option<MCTSOptions>) -> Self {
        let mut options = options.unwrap_or_default();
        options.root_parallelization = if options.root_parallelization == 0 {
            std::thread::available_parallelism()
                .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) })
                .into()
        } else {
            options.root_parallelization
        };
        options.leaf_parallelization = if options.leaf_parallelization == 0 {
            std::thread::available_parallelism()
                .unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) })
                .into()
        } else {
            options.leaf_parallelization
        };

        MCTSPlayer {
            name: format!(
                "{} (R: {}, L: {})",
                name, options.root_parallelization, options.leaf_parallelization
            ),
            policy: ScoredUCTPolicy::new(2f64.sqrt()),
            evaluator: ScoreEvaluator::new(),
            options,
        }
    }
}

#[derive(Clone)]
struct MCTSPlayerSpecification {}
impl MCTSSpecification for MCTSPlayerSpecification {
    type Game = Patchwork;
}

impl Player for MCTSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> Action {
        let search_tree =
            SearchTree::<MCTSPlayerSpecification, ScoredUCTPolicy, ScoreEvaluator>::new(
                game,
                &self.policy,
                &self.evaluator,
                &self.options,
            );
        let result = search_tree.search();
        fs::write("test.txt", search_tree.tree_to_string()).expect("ERROR WRINTING FILE");
        result
    }
}
