use candle_core::error::Result;
use candle_core::Tensor;
use candle_nn::{batch_norm, conv2d, linear, BatchNorm, Conv2d, Conv2dConfig, Module, Sequential, VarBuilder};
use patchwork_core::NaturalActionId;

use crate::resblock::ResBlock;

pub struct ResNet {
    back_bone: Vec<ResBlock>,
    value_head: Sequential,
    policy_head: Sequential,
}

impl ResNet {
    pub fn new(num_res_blocks: usize, num_hidden: usize, vb: VarBuilder) -> Result<Self> {
        //    def __init__(self, game, num_resBlocks, num_hidden):
        //         self.startBlock = nn.Sequential(
        //             nn.Conv2d(3, num_hidden, kernel_size=3, padding=1),
        //             nn.BatchNorm2d(num_hidden),
        //             nn.ReLU()
        //         )

        let back_bone = (0..num_res_blocks)
            .map(|i| ResBlock::new(num_hidden, vb.pp(&format!("resblock_{}", i))))
            .collect::<Result<Vec<_>>>()?;

        //         self.backBone = nn.ModuleList(
        //             [ResBlock(num_hidden) for i in range(num_resBlocks)]
        //         )

        let ph_vb = vb.pp("policy_head");
        let policy_head = candle_nn::seq()
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_all())
            .add(linear(
                3, /* TODO: */
                NaturalActionId::AMOUNT_OF_NORMAL_NATURAL_ACTION_IDS,
                ph_vb.pp("linear"),
            )?);
        drop(ph_vb);

        //         self.policyHead = nn.Sequential(
        //             nn.Conv2d(num_hidden, 32, kernel_size=3, padding=1),
        //             nn.BatchNorm2d(32),
        //             nn.ReLU(),
        //             nn.Flatten(),
        //             nn.Linear(32 * game.row_count * game.column_count, game.action_size)
        //         )

        let vh_vb = vb.pp("policy_head");
        let value_head = candle_nn::seq()
            // .add(conv2d(
            //     Conv2dConfig {
            //         in_channels: num_hidden,
            //         out_channels: 3,
            //         kernel_size: 3,
            //         padding: 1,
            //         ..Default::default()
            //     },
            //     vb.pp("value_head_conv"),
            // )?)
            // .add(batch_norm(3, Default::default(), vb.pp("value_head_bn"))?)
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten_all())
            .add(linear(3 /* TODO: */, 1, vh_vb.pp("linear"))?)
            .add_fn(|xs| xs.tanh());
        drop(vh_vb);

        // 2026

        //         self.valueHead = nn.Sequential(
        //             nn.Conv2d(num_hidden, 3, kernel_size=3, padding=1),
        //             nn.BatchNorm2d(3),
        //             nn.ReLU(),
        //             nn.Flatten(),
        //             nn.Linear(3 * game.row_count * game.column_count, 1),
        //             nn.Tanh()
        //         )
        Ok(Self {
            back_bone,
            value_head,
            policy_head,
        })
    }
}

impl Module for ResNet {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        //         x = self.startBlock(x)
        //         for resBlock in self.backBone:
        //             x = resBlock(x)
        //         policy = self.policyHead(x)
        //         value = self.valueHead(x)
        //         return policy, value

        todo!()
    }
}
