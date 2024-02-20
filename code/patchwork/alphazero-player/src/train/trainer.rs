use candle_core::Device;
use candle_nn::Optimizer;
use tqdm::tqdm;

use crate::{game_state::GameState, PatchZero};

pub struct TrainingArgs {
    // 'C': 2,
    // 'num_searches': 600,
    pub number_of_iterations: u32,           // 8
    pub number_of_self_play_iterations: u32, // 500
    pub number_of_parallel_games: u32,       // 100
    pub number_of_epochs: u32,               // 4

                                             // 'batch_size': 128,
                                             // 'temperature': 1.25,
                                             // 'dirichlet_epsilon': 0.25,
                                             // 'dirichlet_alpha': 0.3
}

pub struct Trainer<
    'a,
    const AMOUNT_PATCH_LAYERS: usize,
    const AMOUNT_RESIDUAL_LAYERS: usize,
    const AMOUNT_FILTERS: usize,
    Optim: Optimizer,
> {
    pub device: &'a Device,
    pub args: TrainingArgs,
    pub optimizer: Optim,
    pub network: PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
}

impl<
        'a,
        const AMOUNT_PATCH_LAYERS: usize,
        const AMOUNT_RESIDUAL_LAYERS: usize,
        const AMOUNT_FILTERS: usize,
        Optim: Optimizer,
    > Trainer<'a, AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS, Optim>
{
    /// Creates a new trainer.
    ///
    /// # Arguments
    ///
    /// * `device` - The device to use for the trainer.
    /// * `network` - The neural network to train.
    /// * `optimizer` - The optimizer to use for training the neural network.
    /// * `args` - The arguments to use for training the neural network.
    pub fn new(
        device: &'a Device,
        network: PatchZero<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
        optimizer: Optim,
        args: TrainingArgs,
    ) -> Self {
        Self {
            device,
            network,
            optimizer,
            args,
        }
    }

    pub fn learn(&mut self) {
        for iteration in 0..self.args.number_of_iterations {
            let mut history = vec![];

            let self_play_iterations = if iteration != self.args.number_of_iterations - 1 {
                self.args.number_of_self_play_iterations / self.args.number_of_parallel_games
            } else {
                self.args.number_of_self_play_iterations - history.len() as u32
            };

            for self_play_iteration in tqdm(0..self_play_iterations) {
                history.extend(self.self_play(false));
            }

            for epoch in tqdm(0..self.args.number_of_epochs) {
                self.train(&mut history);
            }

            // VarMap::load(&mut self, path)
            // VarMap::save(&self, path)
            // AdamW::new()?.
            // safetensors::save(self.optimizer., filename)
            // safetensors::save(tensors, filename)
            // self.network.save
        }
    }

    fn self_play(&self, train: bool) -> Vec<usize> {
        let history = vec![];
        let games = (0..self.args.number_of_parallel_games).map(|_| GameState::default());

        while games.len() > 0 {
            // let states =
        }

        history
    }

    fn train(&self, memory: &mut [usize]) {}
}
