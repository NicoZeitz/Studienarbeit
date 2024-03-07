use candle_core::{DType, Device, IndexOp, Module, Result, Tensor};
use candle_nn::{Linear, VarBuilder};
use patchwork_core::{evaluator_constants, Evaluator, Patchwork, PlayerState, QuiltBoard, StableEvaluator, TurnType};

use lazy_static::lazy_static;

/// Neural Network evaluator.
///
/// # Network Architecture
///
/// ```text
///                  ┌───────────── 32 bit float ─────────────┐
///                          ⬐ σ=ReLU
///         ┌─       ◯ ─── ┌──┐-_
///         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ⎺⎻⎼⎽_ ⬐ σ=ReLU ⬐ σ=ReLU
/// Current │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ ┌──┐ ─── ┌──┐-_
/// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │ ⋱⋰  │  │‾─_
/// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋰⋱  │  │‾─_⟍
///         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋱⋰  │  │-_‾‾─_
///         ┌─       ◯ ─── ┌──┐-_     __── │F │ ⋰⋱  │F │ 32×  ⟍
///         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ──── │ C│ 32× │ C│ ────── ◯ ─ σ(tanh) ⭢ Evaluation
/// Other   │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ │  │ ⋱⋰  │  │⎼⎻⎺_─‾ ⟋
/// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │ ⋰⋱  │  │─‾__─‾
/// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋱⋰  │  │‾_─‾╱
///         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋰⋱  │  │_─‾
/// Flags   ┌─ SP    ◯ ──────────── ◯ ──‾_ └──┘ ─── └──┘-‾
/// (2)     └─ ST    ◯ ──────────── ◯ ⎼⎻⎺
///         2×84+2=170        2×63+2=128    32       32
///       input features      parameters  params   params
/// ```
#[derive(Debug, Clone)]
pub struct NeuralNetworkEvaluator {
    linear_layer_1: Linear,
    linear_layer_2: Linear,
    player_weight: Tensor,
    player_bias: Tensor,
}

lazy_static! {
    static ref ZERO_SCALAR: Tensor = Tensor::zeros((1,), DType::F32, &Device::Cpu).unwrap();
    static ref ONE_SCALAR: Tensor = Tensor::ones((1,), DType::F32, &Device::Cpu).unwrap();
    static ref NEG_ONE_SCALAR: Tensor = Tensor::from_slice(&[-1f32], (1,), &Device::Cpu).unwrap();
    static ref INF_BOUND: Tensor =
        Tensor::from_slice(&[evaluator_constants::POSITIVE_INFINITY as f32], (1,), &Device::Cpu).unwrap();
}

impl Default for NeuralNetworkEvaluator {
    fn default() -> Self {
        // Correct Implementation with loading from safe tensors embedding into exe
        // Temporary code for creating a NeuralNetworkEvaluator
        // let vm = VarMap::new();
        // let vb = VarBuilder::from_varmap(&vm, DType::F32, &Device::Cpu);
        // Self::new(vb)?
        unimplemented!("[NeuralNetworkEvaluator::default]")
    }
}

impl NeuralNetworkEvaluator {
    /// Create a new `NeuralNetworkEvaluator`.
    ///
    /// # Errors
    ///
    /// This function will return an error if the initialization of the network fails.
    #[rustfmt::skip]
    #[allow(clippy::unreadable_literal)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(vb: VarBuilder<'_>) -> Result<Self> {
        let player_weight = vb.get_with_hints((63, 84), "player_weight",  candle_nn::init::DEFAULT_KAIMING_NORMAL)?;
        let player_bias = vb.get_with_hints(63, "player_bias", candle_nn::Init::Uniform {
            lo: -0.1111111111111111, // -1/9
            up: 0.1111111111111111,  //  1/9
        })?;

        let linear_layer_1 = candle_nn::linear(128, 32, vb.pp("linear_1"))?;
        let linear_layer_2 = candle_nn::linear(32, 1, vb.pp("linear_2"))?;

        Ok(Self { linear_layer_1, linear_layer_2, player_weight, player_bias })
    }

    #[allow(clippy::unused_self)]
    fn get_player_tensor(&self, player: &PlayerState) -> Result<Tensor> {
        let mut vec = Vec::with_capacity(84);

        for index in 0..QuiltBoard::TILES {
            vec.push(i32::from(player.quilt_board.get_at(index)) as f32);
        }
        vec.push(f32::from(player.get_position()));
        vec.push(f32::from(player.quilt_board.button_income));
        vec.push(player.button_balance as f32);

        Tensor::from_vec(vec, (84,), &Device::Cpu)
    }

    #[allow(clippy::unused_self)]
    fn get_special_patch_tensor(&self, game: &Patchwork) -> &Tensor {
        if matches!(
            game.turn_type,
            TurnType::SpecialPatchPlacement | TurnType::SpecialPhantom
        ) {
            &ONE_SCALAR
        } else {
            &ZERO_SCALAR
        }
    }

    #[allow(clippy::unused_self)]
    fn get_special_tile_tensor(&self, game: &Patchwork) -> &Tensor {
        if game.is_special_tile_condition_reached_by_player_1() {
            &ONE_SCALAR
        } else if game.is_special_tile_condition_reached_by_player_2() {
            &NEG_ONE_SCALAR
        } else {
            &ZERO_SCALAR
        }
    }

    #[allow(clippy::unused_self)]
    const fn is_player_1(&self, game: &Patchwork) -> bool {
        match game.turn_type {
            TurnType::Normal | TurnType::SpecialPatchPlacement => game.is_player_1(),
            // If we are in a phantom state actually it is the other players turn
            TurnType::NormalPhantom | TurnType::SpecialPhantom => !game.is_player_1(),
        }
    }

    pub fn forward(&self, game: &Patchwork) -> Result<Tensor> {
        let player_1 = self.get_player_tensor(&game.player_1)?;
        let player_2 = self.get_player_tensor(&game.player_2)?;

        // Do the forward pass for the player linear layers
        let forwarded = Tensor::stack(&[&player_1, &player_2], 0)?
            .matmul(&self.player_weight.t()?)?
            .broadcast_add(&self.player_bias)?;

        let forwarded_player_1 = forwarded.i((0, ..))?.relu()?;
        let forwarded_player_2 = forwarded.i((1, ..))?.relu()?;

        let special_patch = self.get_special_patch_tensor(game);
        let special_tile = self.get_special_tile_tensor(game);

        let input_tensor /* 128×1 */ = if self.is_player_1(game) {
            Tensor::cat(&[&forwarded_player_1, &forwarded_player_2, &special_patch, &special_tile], 0)?.unsqueeze(0)?
        }else {
            Tensor::cat(&[&forwarded_player_2, &forwarded_player_1, &special_patch, &special_tile], 0)?.unsqueeze(0)?
        };

        let xs = self.linear_layer_1.forward(&input_tensor)?.clamp(0f32, 127f32)?;

        self.linear_layer_2.forward(&xs)?.squeeze(0)?.sum(0)?.tanh()
    }
}

impl StableEvaluator for NeuralNetworkEvaluator {}
impl Evaluator for NeuralNetworkEvaluator {
    #[rustfmt::skip]
    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32 {
        (self.forward(game).unwrap().to_scalar::<f32>().unwrap() * evaluator_constants::POSITIVE_INFINITY as f32) as i32
    }
}
