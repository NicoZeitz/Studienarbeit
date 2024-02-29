use candle_core::{error::Result, ModuleT, Tensor};
use candle_nn::{batch_norm, conv2d, linear, BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, Linear, VarBuilder};
use patchwork_core::{NaturalActionId, QuiltBoard};

pub struct PolicyHead<const NUMBER_OF_FILTERS: usize> {
    conv: Conv2d,
    batch_norm: BatchNorm,
    linear: Linear,
}

impl<const NUMBER_OF_FILTERS: usize> PolicyHead<NUMBER_OF_FILTERS> {
    const POLICY_HEAD_FILTERS: usize = 2;

    #[allow(clippy::needless_pass_by_value)]
    pub fn new(vb: VarBuilder<'_>) -> Result<Self> {
        let conv = conv2d(
            NUMBER_OF_FILTERS,
            Self::POLICY_HEAD_FILTERS,
            /* kernel size */ 1,
            Conv2dConfig::default(),
            vb.pp("conv"),
        )?;
        let batch_norm = batch_norm(
            Self::POLICY_HEAD_FILTERS,
            BatchNormConfig::default(),
            vb.pp("batch_norm"),
        )?;
        let linear = linear(
            Self::POLICY_HEAD_FILTERS * QuiltBoard::TILES as usize,
            NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,
            vb.pp("linear"),
        )?;
        Ok(Self {
            conv,
            batch_norm,
            linear,
        })
    }
}

impl<const NUMBER_OF_FILTERS: usize> ModuleT for PolicyHead<NUMBER_OF_FILTERS> {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let xs = self.conv.forward_t(xs, train)?;
        let xs = self.batch_norm.forward_t(&xs, train)?;
        let xs = xs.relu()?;
        let xs = xs.flatten_from(1)?;
        self.linear.forward_t(&xs, train)
    }
}
