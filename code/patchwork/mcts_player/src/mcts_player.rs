use evaluator::ScoreEvaluator as Evaluator;
use patchwork_core::{ActionId, Patchwork, Player, PlayerResult};
use tree_policy::ScoredUCTPolicy as TreePolicy;

use crate::{MCTSOptions, SearchTree};

// TODO: report progress to user (win rate, turns taken there, num iterations currently) [debug only]

/// A computer player that uses the Monte Carlo Tree Search (MCTS) algorithm to choose an action.
#[derive(Debug, Clone, PartialEq)]
pub struct MCTSPlayer {
    /// The options for the MCTS algorithm.
    pub options: MCTSOptions,
    /// The name of the player.
    pub name: String,
    /// The policy to select nodes during the selection phase.
    pub policy: TreePolicy,
    /// The evaluator to evaluate the game state.
    pub evaluator: Evaluator,
}

impl MCTSPlayer {
    /// Creates a new [`MCTSPlayer`] with the given name.
    pub fn new(name: impl Into<String>, options: Option<MCTSOptions>) -> Self {
        let options = options.unwrap_or_default();
        MCTSPlayer {
            name: format!(
                "{} (R: {}, L: {})",
                name.into(),
                options.root_parallelization,
                options.leaf_parallelization
            ),
            policy: Default::default(),
            evaluator: Default::default(),
            options,
        }
    }
}

impl Default for MCTSPlayer {
    fn default() -> Self {
        Self::new("MCTS Player".to_string(), Default::default())
    }
}

#[derive(Clone)]
struct MCTSPlayerSpecification {}

impl Player for MCTSPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let search_tree = SearchTree::<TreePolicy, Evaluator>::new(game, &self.policy, &self.evaluator, &self.options);
        Ok(search_tree.search())
        // fs::write("test.txt", search_tree.tree_to_string()).expect("ERROR WRITING FILE"); // TODO: remove
    }
}
