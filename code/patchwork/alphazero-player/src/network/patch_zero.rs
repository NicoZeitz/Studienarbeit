use candle_core::{Device, Result, Tensor};
use candle_nn::VarBuilder;
use patchwork_core::Patchwork;

use crate::network::{game_encoder::GameEncoder, resnet::ResNet};

/// The neural network that plays patchwork.
#[allow(dead_code)]
pub struct PatchZero<
    // The 3 patches that can be taken are encoded into separate layers.
    const AMOUNT_PATCH_LAYERS: usize = 3,
    // 40 in paper
    const AMOUNT_RESIDUAL_LAYERS: usize = 20,
    // 256 in paper
    const AMOUNT_FILTERS: usize = 128,
> {
    device: Device,
    encoder: GameEncoder<AMOUNT_PATCH_LAYERS>,
    network: ResNet<AMOUNT_PATCH_LAYERS, AMOUNT_RESIDUAL_LAYERS, AMOUNT_FILTERS>,
}

impl<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_RESIDUAL_LAYERS: usize, const NUMBER_OF_FILTERS: usize>
    PatchZero<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
{
    /// Creates a new patch zero neural network.
    ///
    /// # Arguments
    ///
    /// * `vb` - The variable builder to use for creating the variables.
    /// * `device` - The device to use for the variables.
    ///
    /// # Returns
    ///
    /// A new patch zero neural network.
    #[allow(dead_code)]
    pub fn new(vb: VarBuilder, device: Device) -> Result<Self> {
        let encoder = GameEncoder::<NUMBER_OF_PATCH_LAYERS>::new(vb.pp("encoder"), device.clone())?;
        let network =
            ResNet::<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>::new(vb.pp("network"))?;

        Ok(Self {
            encoder,
            network,
            device,
        })
    }

    /// Forward propagates the neural network.
    ///
    /// # Arguments
    ///
    /// * `game` - The game to forward propagate.
    /// * `train` - Whether to train the neural network.
    ///
    /// # Returns
    ///
    /// The policy and value tensors.
    #[allow(dead_code)]
    pub fn forward_t(&self, games: &[&Patchwork], train: bool) -> Result<(Tensor, Tensor)> {
        let stack = self.encoder.encode_state(games)?;
        self.network.forward_t(&stack, train)
    }
}
