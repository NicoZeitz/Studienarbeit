use candle_core::{Device, Result, Tensor};
use candle_nn::VarBuilder;
use patchwork_core::Patchwork;

use crate::game_encoding::GameEncoder;

pub struct PatchZero<'a> {
    device: &'a Device,
    encoder: GameEncoder<'a>,
    // network: ResNet,
}

impl<'a> PatchZero<'a> {
    const NORMAL_PATCH_LAYERS: usize = 3;

    pub fn new(vb: VarBuilder, device: &Device) -> Result<Self> {
        let encoder = GameEncoder::new(vb.pp("encoder"), device)?;
        // let network = ResNet::new()

        Ok(Self { encoder, device })
    }

    pub fn forward(&self, game: &Patchwork) -> Result<(Tensor, Tensor)> {
        let stack = self.encoder.encode_state(game)?;

        todo!()
    }
}
