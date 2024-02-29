use crate::network::PatchZero;

// The amount of layers to use for encoding patches inside the neural network as own blocks.
// The minimum that should be used is 3 so that each available patch can be encoded in it's own
// layer.
pub const DEFAULT_AMOUNT_PATCH_LAYERS: usize = 3;
// The amount of residual layers to use in the neural network.
//
// # Note
//
// In the AlphaGo Zero paper, the amount of residual layers is 40.
pub const DEFAULT_AMOUNT_RESIDUAL_LAYERS: usize = 10;
// The amount of filters to use in the neural network.
//
// # Note
//
// In the AlphaGo Zero paper, the amount of filters is 256.
pub const DEFAULT_AMOUNT_FILTERS: usize = 64;

pub type DefaultPatchZero =
    PatchZero<DEFAULT_AMOUNT_PATCH_LAYERS, DEFAULT_AMOUNT_RESIDUAL_LAYERS, DEFAULT_AMOUNT_FILTERS>;
