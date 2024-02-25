use candle_core::{error::Result, ModuleT, Tensor};
use candle_nn::{batch_norm, conv2d, BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, VarBuilder};

pub struct ConvolutionalLayer<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_FILTERS: usize> {
    conv: Conv2d,
    batch_norm: BatchNorm,
}

impl<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_FILTERS: usize>
    ConvolutionalLayer<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_FILTERS>
{
    pub fn new(vb: VarBuilder) -> Result<Self> {
        let conv2d_config: Conv2dConfig = Conv2dConfig {
            padding: 1,
            ..Conv2dConfig::default()
        };

        let conv = conv2d(
            NUMBER_OF_PATCH_LAYERS + 5,
            NUMBER_OF_FILTERS,
            /* kernel size */ 3,
            conv2d_config,
            vb.pp("conv"),
        )?;
        let batch_norm = batch_norm(NUMBER_OF_FILTERS, BatchNormConfig::default(), vb.pp("batch_norm"))?;
        Ok(Self { conv, batch_norm })
    }
}

impl<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_FILTERS: usize> ModuleT
    for ConvolutionalLayer<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_FILTERS>
{
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let xs = self.conv.forward_t(xs, train)?;
        let xs = self.batch_norm.forward_t(&xs, train)?;
        xs.relu()
    }
}
