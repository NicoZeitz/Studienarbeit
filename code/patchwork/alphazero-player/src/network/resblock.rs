use candle_core::Tensor;
use candle_core::{error::Result, ModuleT};
use candle_nn::{batch_norm, conv2d, BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, Module, VarBuilder};

use crate::network::squeeze_excitation::SqueezeExcitation;

pub struct ResBlock {
    convolution_1: Conv2d,
    batch_norm_1: BatchNorm,
    convolution_2: Conv2d,
    batch_norm_2: BatchNorm,
    squeeze_excitation: SqueezeExcitation,
}

impl ResBlock {
    pub fn new(num_filters: usize, vb: VarBuilder) -> Result<Self> {
        let convolution_config = Conv2dConfig {
            padding: 1,
            ..Conv2dConfig::default()
        };

        let convolution_1 = conv2d(num_filters, num_filters, 3, convolution_config, vb.pp("conv1"))?;
        let batch_norm_1 = batch_norm(num_filters, BatchNormConfig::default(), vb.pp("batch_norm1"))?;
        let convolution_2 = conv2d(num_filters, num_filters, 3, convolution_config, vb.pp("conv2"))?;
        let batch_norm_2 = batch_norm(num_filters, BatchNormConfig::default(), vb.pp("batch_norm1"))?;
        let squeeze_excitation =
            SqueezeExcitation::new(num_filters, num_filters / 8, None, None, vb.pp("squeeze_excitation"))?; // TODO: get 8 from NUMBER_OF_PATCH_LAYERS + 5

        Ok(Self {
            convolution_1,
            batch_norm_1,
            convolution_2,
            batch_norm_2,
            squeeze_excitation,
        })
    }
}

impl ModuleT for ResBlock {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let residual = xs;
        let mut xs = self.convolution_1.forward(xs)?;
        xs = self.batch_norm_1.forward_t(&xs, train)?;
        xs = xs.relu()?;
        xs = self.convolution_2.forward_t(&xs, train)?;
        xs = self.batch_norm_2.forward_t(&xs, train)?;
        xs = self.squeeze_excitation.forward_t(&xs, train)?;
        xs = (xs + residual)?;
        xs.relu()
    }
}
