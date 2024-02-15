use candle_core::{Device, Result, Tensor};
use patchwork_core::{ActionId, NaturalActionId};

// Vec<ActionId> -> Vec<NaturalActionid>

// 2026

pub fn convert_action_ids_to_possible_natural_actions(
    action_ids: &[ActionId],
    device: &Device,
) -> Result<(Tensor, Vec<Option<ActionId>>)> {
    let mut values = [0.0; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];
    let mut corresponding_action_id = [None; NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS];

    for action_id in action_ids {
        let natural_action_id = action_id.to_natural_action_id().as_bits() as usize;

        values[natural_action_id] = 1.0;
        corresponding_action_id[natural_action_id] = Some(*action_id);
    }

    Ok((
        Tensor::from_vec(values.to_vec(), (values.len(),), &device)?,
        corresponding_action_id.to_vec(),
    ))
}
