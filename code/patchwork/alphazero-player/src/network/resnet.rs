use candle_core::error::Result;
use candle_core::Tensor;
use candle_nn::{batch_norm, conv2d, linear, BatchNormConfig, Conv2dConfig, ModuleT, VarBuilder};
use patchwork_core::{NaturalActionId, QuiltBoard};

use crate::network::{resblock::ResBlock, sequential::*};

pub struct ResNet<
    const NUMBER_OF_PATCH_LAYERS: usize,
    const NUMBER_OF_RESIDUAL_LAYERS: usize,
    const NUMBER_OF_FILTERS: usize,
> {
    convolutional_layer: Sequential,
    residual_layers: Vec<ResBlock>,
    value_head: Sequential,
    policy_head: Sequential,
}

// https://adspassets.blob.core.windows.net/website/content/alpha_go_zero_cheat_sheet.png
// https://discovery.ucl.ac.uk/id/eprint/10045895/1/agz_unformatted_nature.pdf
impl<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_RESIDUAL_LAYERS: usize, const NUMBER_OF_FILTERS: usize>
    ResNet<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
{
    const POLICY_HEAD_FILTERS: usize = 2;
    const VALUE_HEAD_FILTERS: usize = 1;

    #[rustfmt::skip]
    pub fn new(vb: VarBuilder) -> Result<Self> {
        let conv2d_config: Conv2dConfig = Conv2dConfig {
            padding: 1,
            ..Conv2dConfig::default()
        };

        let convolutional_vb = vb.pp("convolutional_layer");
        let convolutional_layer = seq()
            .add(conv2d(NUMBER_OF_PATCH_LAYERS + 5, NUMBER_OF_FILTERS, /* kernel size */ 3, conv2d_config, convolutional_vb.pp("conv"))?)
            .add(batch_norm(NUMBER_OF_FILTERS, BatchNormConfig::default(), convolutional_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu());
        drop(convolutional_vb);

        let residual_layers = (0..NUMBER_OF_RESIDUAL_LAYERS)
            .map(|i| ResBlock::new(NUMBER_OF_FILTERS, vb.pp(format!("resblock_{}", i).as_str())))
            .collect::<Result<Vec<_>>>()?;

        let policy_head_vb = vb.pp("policy_head");
        let policy_head = seq()
            .add(conv2d(NUMBER_OF_FILTERS, Self::POLICY_HEAD_FILTERS, /* kernel size */ 1, Conv2dConfig::default(), policy_head_vb.pp("conv"))?)
            .add(batch_norm(Self::POLICY_HEAD_FILTERS, BatchNormConfig::default(), policy_head_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_from(1))
            .add(linear(Self::POLICY_HEAD_FILTERS * QuiltBoard::TILES as usize, NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS, policy_head_vb.pp("linear"))?);
        drop(policy_head_vb);

        let value_head_vb = vb.pp("value_head");
        let value_head = seq()
            .add(conv2d(NUMBER_OF_FILTERS, Self::VALUE_HEAD_FILTERS, /* kernel size */ 1, Conv2dConfig::default(), value_head_vb.pp("conv"))?)
            .add(batch_norm(Self::VALUE_HEAD_FILTERS, BatchNormConfig::default(), value_head_vb.pp("batch_norm"))?)
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_from(1))
            .add(linear(Self::VALUE_HEAD_FILTERS * QuiltBoard::TILES as usize, 256, value_head_vb.pp("linear_1"))?)
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

    pub fn forward_t(&self, xs: &Tensor, train: bool) -> Result<(Tensor, Tensor)> {
        let mut xs = self.convolutional_layer.forward_t(xs, train)?;
        for res_block in &self.residual_layers {
            xs = res_block.forward_t(&xs, train)?;
        }
        let policies = self.policy_head.forward_t(&xs, train)?;
        let values = self.value_head.forward_t(&xs, train)?.squeeze(1)?;
        Ok((policies, values))
    }
}
