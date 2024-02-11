use candle_core::Tensor;
use candle_core::{error::Result, ModuleT};
use candle_nn::{conv2d, Conv2d, Conv2dConfig, VarBuilder};

/// Squeeze and Excitation block
///
/// Paper: https://arxiv.org/abs/1709.01507
///
/// Ported from [Pytorch SqueezeExcitation](https://pytorch.org/vision/main/_modules/torchvision/ops/misc.html#SqueezeExcitation)
pub struct SqueezeExcitation {
    fc1: Conv2d,
    fc2: Conv2d,
    activation: Box<dyn ModuleT>,
    scale_activation: Box<dyn ModuleT>,
}

impl SqueezeExcitation {
    pub fn new(
        input_channels: usize,
        squeeze_channels: usize,
        activation: Option<Box<dyn ModuleT>>,
        scale_activation: Option<Box<dyn ModuleT>>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let fc1 = conv2d(
            input_channels,
            squeeze_channels,
            1,
            Conv2dConfig::default(),
            vb.pp("fc1"),
        )?;
        let fc2 = conv2d(
            squeeze_channels,
            input_channels,
            1,
            Conv2dConfig::default(),
            vb.pp("fc2"),
        )?;
        let activation = activation.unwrap_or(Box::new(candle_nn::func_t(|xs, _train| xs.relu())));
        let scale_activation =
            scale_activation.unwrap_or(Box::new(candle_nn::func_t(|xs, _train| candle_nn::ops::sigmoid(xs))));

        Ok(Self {
            fc1,
            fc2,
            activation,
            scale_activation,
        })
    }

    fn scale(&self, input: &Tensor, train: bool) -> Result<Tensor> {
        let scale = input.avg_pool2d(1)?;
        let scale = self.fc1.forward_t(&scale, train)?;
        let scale = self.activation.forward_t(&scale, train)?;
        let scale = self.fc2.forward_t(&scale, train)?;
        let scale = self.scale_activation.forward_t(&scale, train)?;
        Ok(scale)
    }
}

impl ModuleT for SqueezeExcitation {
    fn forward_t(&self, input: &Tensor, train: bool) -> Result<Tensor> {
        let scale = self.scale(input, train)?;
        scale * input
    }
}
