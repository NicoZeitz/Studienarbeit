use std::{env, fs, path::Path};

use alphazero_player::{network::DefaultPatchZero, train::{Trainer, TrainingArgs}};

use candle_core::{DType, Device};
use candle_nn::{VarBuilder, VarMap};
use patchwork_core::{Patchwork, PlayerResult};
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

#[allow(dead_code)]
fn test_network() -> PlayerResult<()> {
    let varmap = VarMap::new();
    let var_builder = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);
    let network = DefaultPatchZero::new(var_builder, Device::Cpu)?;
    let state = Patchwork::get_initial_state(None);

    let start_time = std::time::Instant::now();
    network.forward_t(&[&state], false)?;
    println!("Took: {:?}", start_time.elapsed());

    Ok(())
}