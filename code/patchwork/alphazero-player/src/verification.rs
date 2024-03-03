use std::{env, fs, path::Path};

use alphazero_player::train::{Trainer, TrainingArgs};

use candle_core::Device;
use patchwork_core::PlayerResult;
use tree_policy::PUCTPolicy;

fn main() -> PlayerResult<()> {
    let training_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("training");
    let training_directory = training_directory.as_path();

    fs::create_dir_all(training_directory)?;

    env::set_var(
        "RUST_BACKTRACE",
        env::var("RUST_BACKTRACE").map_or_else(|_| "1".to_string(), |s| s),
    );

    let trainer = Trainer::new(
        training_directory,
        TrainingArgs {
            number_of_training_iterations: 1000,
            number_of_mcts_iterations: 600,
            number_of_parallel_games: 10,
            number_of_self_play_iterations: 10,
            number_of_epochs: 5,
            // change later
            batch_size: 128,
            learning_rate: 0.01,
            ..TrainingArgs::default()
        },
        Device::Cpu,
        // Device::cuda_if_available(0).unwrap_or(Device::Cpu),
    );

    let start_time = std::time::Instant::now();
    trainer.learn::<PUCTPolicy>()?;
    println!("Took: {:?}", start_time.elapsed());

    Ok(())
}
