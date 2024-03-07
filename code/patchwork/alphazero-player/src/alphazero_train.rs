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
        TrainingArgs::default(),
        Device::Cpu, // Device::cuda_if_available(0).unwrap_or(Device::Cpu),
    )?;

    let start_time = std::time::Instant::now();
    trainer.learn::<PUCTPolicy>()?;
    println!("Took: {:?}", start_time.elapsed());

    Ok(())
}
