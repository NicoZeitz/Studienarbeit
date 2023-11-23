use patchwork_core::{Action, Patchwork, Player};

use crate::{mcts_player::search_tree::SearchTree, MCTSOptions, UCTPolicy, WinLossEvaluator};

use crate::MCTSSpecification;

/// A player that uses Monte Carlo Tree Search to select actions.
pub struct MCTSPlayer {
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The name of the player.
    pub name: String,
    /// The policy to select nodes during the selection phase.
    pub policy: UCTPolicy,
    /// The evaluator to evaluate the game state.
    pub evaluator: WinLossEvaluator,
}

impl MCTSPlayer {
    /// Creates a new [`MCTSPlayer`].
    pub fn new(name: String, options: Option<MCTSOptions>) -> Self {
        MCTSPlayer {
            name,
            policy: UCTPolicy::new(1.0),
            evaluator: WinLossEvaluator::new(),
            options: options.unwrap_or_default(),
        }
    }
}

struct MCTSPlayerSpecification {}
impl MCTSSpecification for MCTSPlayerSpecification {
    type Game = Patchwork;
}

impl Player for MCTSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> Action {
        let search_tree = SearchTree::<MCTSPlayerSpecification, UCTPolicy, WinLossEvaluator>::new(
            game,
            &self.policy,
            &self.evaluator,
            &self.options,
        );
        search_tree.search()
    }
}
