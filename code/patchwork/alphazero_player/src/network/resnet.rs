use candle_core::error::Result;
use candle_core::Tensor;
use candle_nn::{batch_norm, conv2d, linear, BatchNormConfig, Conv2dConfig, Module, Sequential, VarBuilder};
use patchwork_core::NaturalActionId;

use crate::network::ResBlock;

#[allow(dead_code)]
pub struct ResNet {
    convolutional_layer: Sequential,
    residual_layers: Vec<ResBlock>,
    value_head: Sequential,
    policy_head: Sequential,
}

// https://adspassets.blob.core.windows.net/website/content/alpha_go_zero_cheat_sheet.png
// https://discovery.ucl.ac.uk/id/eprint/10045895/1/agz_unformatted_nature.pdf
impl ResNet {
    const POLICY_HEAD_FILTERS: usize = 2;
    const VALUE_HEAD_FILTERS: usize = 1;

    #[rustfmt::skip]
    #[allow(dead_code)]
    pub fn new(
        game_state: /* 9x9xN */ &Tensor,
        number_of_residual_layers: usize,
        num_filters: usize,
        vb: VarBuilder,
    ) -> Result<Self> {
        let conv2d_config: Conv2dConfig = Conv2dConfig {
            padding: 1,
            ..Conv2dConfig::default()
        };

        let convolutional_vb = vb.pp("convolutional_layer");
        let convolutional_layer = candle_nn::seq()
            .add(conv2d(game_state.dim(2)?, num_filters, /* kernel size */ 3, conv2d_config, convolutional_vb.pp("conv"))?)
            .add(batch_norm(num_filters, BatchNormConfig::default(), convolutional_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu());
        drop(convolutional_vb);

        let residual_layers = (0..number_of_residual_layers)
            .map(|i| ResBlock::new(num_filters, vb.pp(format!("resblock_{}", i).as_str())))
            .collect::<Result<Vec<_>>>()?;

        let policy_head_vb = vb.pp("policy_head");
        let policy_head = candle_nn::seq()
            .add(conv2d(num_filters, Self::POLICY_HEAD_FILTERS, /* kernel size */ 1, Conv2dConfig::default(), policy_head_vb.pp("conv"))?)
            .add(batch_norm(Self::POLICY_HEAD_FILTERS, BatchNormConfig::default(), policy_head_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_all())
            .add(linear(Self::POLICY_HEAD_FILTERS * game_state.dim(0)? * game_state.dim(1)?, NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS, policy_head_vb.pp("linear"))?);
        drop(policy_head_vb);

        let value_head_vb = vb.pp("value_head");
        let value_head = candle_nn::seq()
            .add(conv2d(num_filters, Self::VALUE_HEAD_FILTERS, /* kernel size */ 1, Conv2dConfig::default(), value_head_vb.pp("conv"))?)
            .add(batch_norm(Self::VALUE_HEAD_FILTERS, BatchNormConfig::default(), value_head_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_all())
            .add(linear(Self::VALUE_HEAD_FILTERS * game_state.dim(0)? * game_state.dim(1)?, 256, value_head_vb.pp("linear_1"))?)
            .add_fn(|xs| xs.relu())
            .add(linear(256, 1, value_head_vb.pp("linear_2"))?)
            .add_fn(|xs| xs.tanh());
        drop(value_head_vb);

        Ok(Self {
            convolutional_layer,
            residual_layers,
            value_head,
            policy_head,
        })
    }

    #[allow(dead_code)]
    pub fn forward(&self, xs: &Tensor) -> Result<(Tensor, Tensor)> {
        let mut xs = self.convolutional_layer.forward(xs)?;
        for res_block in &self.residual_layers {
            xs = res_block.forward(&xs)?;
        }
        let policy = self.policy_head.forward(&xs)?;
        let value = self.value_head.forward(&xs)?;
        Ok((policy, value))
    }
}
