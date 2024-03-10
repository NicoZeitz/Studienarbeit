use std::{env, fs, path::Path};

use patchwork_core::PlayerResult;

use crate::{trainer::Trainer, training_args::TrainingArgs};

mod trainer;
mod training_args;

pub fn main() -> PlayerResult<()> {
    let training_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("training");
    let training_directory = training_directory.as_path();

    fs::create_dir_all(training_directory)?;

    env::set_var(
        "RUST_BACKTRACE",
        env::var("RUST_BACKTRACE").map_or_else(|_| "1".to_string(), |s| s),
    );

    let trainer = Trainer::new(training_directory, TrainingArgs::default());

    trainer.learn()?;

    Ok(())
}
