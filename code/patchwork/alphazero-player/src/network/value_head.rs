use candle_core::{error::Result, ModuleT, Tensor};
use candle_nn::{batch_norm, conv2d, linear, BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, Linear, VarBuilder};
use patchwork_core::QuiltBoard;

pub struct ValueHead<const NUMBER_OF_FILTERS: usize> {
    conv: Conv2d,
    batch_norm: BatchNorm,
    linear_1: Linear,
    linear_2: Linear,
}

impl<const NUMBER_OF_FILTERS: usize> ValueHead<NUMBER_OF_FILTERS> {
    const VALUE_HEAD_FILTERS: usize = 1;

    pub fn new(vb: VarBuilder) -> Result<Self> {
        let conv = conv2d(
            NUMBER_OF_FILTERS,
            Self::VALUE_HEAD_FILTERS,
            /* kernel size */ 1,
            Conv2dConfig::default(),
            vb.pp("conv"),
        )?;
        let batch_norm = batch_norm(
            Self::VALUE_HEAD_FILTERS,
            BatchNormConfig::default(),
            vb.pp("batch_norm"),
        )?;
        let linear_1 = linear(
            Self::VALUE_HEAD_FILTERS * QuiltBoard::TILES as usize,
            256,
            vb.pp("linear_1"),
        )?;
        let linear_2 = linear(256, 1, vb.pp("linear_2"))?;
        Ok(Self {
            conv,
            batch_norm,
            linear_1,
            linear_2,
        })
    }
}

impl<const NUMBER_OF_FILTERS: usize> ModuleT for ValueHead<NUMBER_OF_FILTERS> {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let xs = self.conv.forward_t(xs, train)?;
        let xs = self.batch_norm.forward_t(&xs, train)?;
        let xs = xs.relu()?;
        let xs = xs.flatten_from(1)?;
        let xs = self.linear_1.forward_t(&xs, train)?;
        let xs = xs.relu()?;
        let xs = self.linear_2.forward_t(&xs, train)?;
        xs.tanh()
    }
}
