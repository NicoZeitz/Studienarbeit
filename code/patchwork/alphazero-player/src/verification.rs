use std::path::Path;

use alphazero_player::train::{Trainer, TrainingArgs};

use candle_core::Device;
use patchwork_core::PlayerResult;
use tree_policy::PUCTPolicy;

fn main() -> PlayerResult<()> {
    let trainer = Trainer::new(
        Path::new("training"),
        TrainingArgs {
            number_of_training_iterations: 1,
            number_of_epochs: 1,
            ..TrainingArgs::default()
        },
        Device::cuda_if_available(0)?,
    );
    trainer.learn::<PUCTPolicy>()?;

    Ok(())
}
