pub use action_orderer::*;
pub use patchwork_core::{
    status_flags, time_board_flags, Action, ActionId, GameOptions, NaturalActionId, Notation, Patch, PatchManager,
    PatchTransformation, Patchwork, PatchworkError, PlayerState, QuiltBoard, Termination, TerminationType, TimeBoard,
};

pub mod evaluator {
    pub use evaluator::*;
    pub use patchwork_core::{Evaluator, StableEvaluator};
}

pub mod player {
    pub use alphazero_player::*;
    pub use greedy_player::*;
    pub use human_player::*;
    pub use mcts_player::*;
    pub use minimax_player::*;
    pub use patchwork_core::{Logging, Player};
    pub use principal_variation_search_player::*;
    pub use random_player::*;
}

pub mod tree_policy {
    pub use patchwork_core::{TreePolicy, TreePolicyNode};
    pub use tree_policy::*;
}

pub mod prelude {
    pub use super::evaluator::*;
    pub use super::player::*;
    pub use super::tree_policy::*;
    pub use patchwork_core::{ActionId, Patch, Patchwork, Termination, TerminationType};
}

mod game_manager;

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use ::evaluator::StaticEvaluator;

    use super::player::*;
    use super::*;

    #[test]
    fn random_player() {
        let player = Box::new(RandomPlayer::new("Random Player", Some(RandomOptions::default())));
        test_player(player);
    }

    #[test]
    fn greedy_player() {
        let player: GreedyPlayer = GreedyPlayer::new("Greedy Player");
        let player = Box::new(player);
        test_player(player);
    }

    #[test]
    fn minimax_player() {
        let player = Box::new(MinimaxPlayer::<StaticEvaluator>::new(
            "Minimax Player",
            Some(MinimaxOptions {
                depth: 3,
                amount_actions_per_piece: 3,
            }),
        ));
        test_player(player);
    }

    #[test]
    #[ignore = "PVS Player fails, needs to be investigated (maybe because of short time?)"]
    fn pvs_player() {
        let player: Box<dyn Player> = DefaultPVSPlayer::<TableActionOrderer, StaticEvaluator>::new(
            "PVS Player",
            Some(PVSOptions {
                logging: Logging::Disabled,
                time_limit: std::time::Duration::from_secs(1),
                features: PVSFeatures::default(),
            }),
        );
        test_player(player);
    }

    #[test]
    fn mcts_player() {
        let player: MCTSPlayer = MCTSPlayer::new(
            "MCTS Player",
            Some(MCTSOptions {
                end_condition: MCTSEndCondition::Time(std::time::Duration::from_secs(1)),
                reuse_tree: true,
                leaf_parallelization: NonZeroUsize::new(1).unwrap(),
                root_parallelization: NonZeroUsize::new(1).unwrap(),
                logging: Logging::Disabled,
            }),
        );
        let player = Box::new(player);
        test_player(player);
    }

    #[test]
    #[ignore = "AlphaZero player is not yet implemented"]
    fn alphazero_player() {
        let player: AlphaZeroPlayer = AlphaZeroPlayer::new(
            "AlphaZero Player",
            Some(AlphaZeroOptions {
                end_condition: AlphaZeroEndCondition::Time {
                    duration: std::time::Duration::from_secs(1),
                    safety_margin: std::time::Duration::from_millis(50),
                },
                logging: Logging::Disabled,
                ..Default::default()
            }),
        );
        let player = Box::new(player);
        test_player(player);
    }

    fn test_player(mut player: Box<dyn Player>) {
        let mut state = Patchwork::get_initial_state(Some(GameOptions { seed: 42 }));
        loop {
            let action_result = player.get_action(&state);

            let action = match action_result {
                Ok(action) => action,
                Err(error) => {
                    println!("Player '{}' get_action failed with: {}", player.name(), error);
                    println!("State:\n{state}");
                    panic!("{}", error);
                }
            };

            let valid_actions = state.get_valid_actions();
            if !valid_actions.contains(&action) {
                println!("Player '{}' chose invalid action: {}", player.name(), action);
                println!("State:\n{state}");
                panic!("Invalid action!");
            }

            match state.do_action(action, false) {
                Ok(()) => {}
                Err(error) => {
                    println!("Player '{}' do_action failed with: {}", player.name(), error);
                    println!("State:\n{state}");
                    panic!("{}", error);
                }
            }

            if state.is_terminated() {
                break;
            }
        }
    }
}
