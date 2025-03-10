use candle_core::{Device, Result, Tensor};
use candle_nn::VarBuilder;
use patchwork_core::Patchwork;

use crate::network::{game_encoder::GameEncoder, resnet::ResNet};

/// The neural network that plays patchwork.
#[derive(Debug, Clone)]
pub struct PatchZero<const AMOUNT_PATCH_LAYERS: usize, const AMOUNT_RESIDUAL_LAYERS: usize, const AMOUNT_FILTERS: usize>
{
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
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(vb: VarBuilder<'_>, device: Device) -> Result<Self> {
        let encoder = GameEncoder::<NUMBER_OF_PATCH_LAYERS>::new(vb.pp("encoder"), device)?;
        let network =
            ResNet::<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>::new(vb.pp("network"))?;

        Ok(Self { encoder, network })
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
    ///
    /// # Errors
    ///
    /// When there is a error inside the neural network.
    pub fn forward_t(&self, games: &[&Patchwork], train: bool) -> Result<(Tensor, Tensor)> {
        let stack = self.encoder.encode_state(games)?;
        self.network.forward_t(&stack, train)
    }
}
