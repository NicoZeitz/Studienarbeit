use candle_core::error::Result;
use candle_core::Tensor;
use candle_nn::{ModuleT, VarBuilder};

use crate::network::{
    convolutional_layer::ConvolutionalLayer, policy_head::PolicyHead, resblock::ResBlock, value_head::ValueHead,
};

pub struct ResNet<
    const NUMBER_OF_PATCH_LAYERS: usize,
    const NUMBER_OF_RESIDUAL_LAYERS: usize,
    const NUMBER_OF_FILTERS: usize,
> {
    convolutional_layer: ConvolutionalLayer<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_FILTERS>,
    residual_layers: Vec<ResBlock>,
    value_head: ValueHead<NUMBER_OF_FILTERS>,
    policy_head: PolicyHead<NUMBER_OF_FILTERS>,
}

// https://adspassets.blob.core.windows.net/website/content/alpha_go_zero_cheat_sheet.png
// https://discovery.ucl.ac.uk/id/eprint/10045895/1/agz_unformatted_nature.pdf
impl<const NUMBER_OF_PATCH_LAYERS: usize, const NUMBER_OF_RESIDUAL_LAYERS: usize, const NUMBER_OF_FILTERS: usize>
    ResNet<NUMBER_OF_PATCH_LAYERS, NUMBER_OF_RESIDUAL_LAYERS, NUMBER_OF_FILTERS>
{
    #[rustfmt::skip]
    pub fn new(vb: VarBuilder) -> Result<Self> {
        let convolutional_layer = ConvolutionalLayer::new(vb.pp("convolutional_layer"))?;
        let residual_layers = (0..NUMBER_OF_RESIDUAL_LAYERS)
            .map(|i| ResBlock::new(NUMBER_OF_FILTERS, vb.pp(format!("resblock_{}", i).as_str())))
            .collect::<Result<Vec<_>>>()?;
        let policy_head = PolicyHead::new(vb.pp("policy_head"))?;
        let value_head = ValueHead::new(vb.pp("value_head"))?;

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
