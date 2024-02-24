use std::collections::VecDeque;

use candle_core::{Device, Result, Tensor};
use patchwork_core::{ActionId, NaturalActionId, Patchwork};

/// Maps the given games to action tensors. This means that for each game the available actions are
/// mapped to natural action ids. Then a tensor is created with each row representing a game and
/// each column representing one of all natural action ids. The tensor is filled with 1.0 at the
/// positions of the available actions and 0.0 at the positions of the unavailable actions.
/// Additionally a list of corresponding surrogate action ids is created for each game.
///
/// # Arguments
///
/// * `games` - The games to map to action tensors.
/// * `device` - The device to use for the tensor.
///
/// # Returns
///
/// The action tensor and the corresponding action ids for each game.
pub fn map_games_to_action_tensors(
    games: &[&Patchwork],
    device: &Device,
) -> Result<(
    Tensor,
    VecDeque<[ActionId; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS]>,
)> {
    let mut values = vec![0f32; games.len() * NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];
    let mut corresponding_action_ids =
        VecDeque::<[ActionId; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS]>::with_capacity(games.len());

    for (game_index, game) in games.iter().enumerate() {
        let mut game_corresponding_action_ids =
            [ActionId::null(); NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];

        for action_id in game.get_valid_actions() {
            let natural_action_id = action_id.to_natural_action_id().as_bits() as usize;

            values[game_index * NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS + natural_action_id] = 1.0;
            game_corresponding_action_ids[natural_action_id] = action_id;
        }

        corresponding_action_ids.push_front(game_corresponding_action_ids);
    }

    Ok((
        Tensor::from_vec(
            values,
            (games.len(), NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS),
            device,
        )?,
        corresponding_action_ids,
    ))
}
