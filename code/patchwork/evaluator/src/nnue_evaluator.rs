use candle_core::{DType, Device, IndexOp, Module, Result, Tensor};
use candle_nn::{Linear, VarBuilder};
use patchwork_core::{
    evaluator_constants, ActionId, Evaluator, Patchwork, PlayerState, QuiltBoard, StableEvaluator, TurnType,
};

/// ƎUИИ (Efficiently Updatable Neural Network) evaluator.
///
/// # Network Architecture
///
///
/// ```text
///                  ┌───────────── 32 bit float ─────────────┐
///                          ⬐ σ=Clipped ReLU(0, 127)
///         ┌─       ◯ ─── ┌──┐-_
///         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ⎺⎻⎼⎽_ ┌─────────┬─── σ=Clipped ReLU(0, 127)
/// Current │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ ┌──┐ ─── ┌──┐-_
/// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │ ⋱⋰  │  │‾─_
/// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋰⋱  │  │‾─_⟍
///         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋱⋰  │  │-_‾‾─_
///         ┌─       ◯ ─── ┌──┐-_     __── │F │ ⋰⋱  │F │ 32×  ⟍     ⬐ Clip(Min, Max)
///         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ──── │ C│ 32× │ C│ ────── ◯ ─ σ ⭢ Evaluation
/// Other   │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ │  │ ⋱⋰  │  │⎼⎻⎺_─‾ ⟋
/// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │ ⋰⋱  │  │─‾__─‾
/// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋱⋰  │  │‾_─‾╱
///         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋰⋱  │  │_─‾
/// Flags   ┌─ SP    ◯ ──────────── ◯ ──‾_ └──┘ ─── └──┘-‾
/// (2)     └─ ST    ◯ ──────────── ◯ ⎼⎻⎺
///         2×84+2=170        2×63+2=128    32       32
///       input features      parameters  params   params
///                  └──────────────┘   └─────────────────────┘
///                 incremental update    normal forward pass
/// ```
#[derive(Debug, Clone)]
pub struct NNUEEvaluator {
    scale_scalar: Tensor,       // 1×1
    zero_scalar: Tensor,        // 1×1
    one_scalar: Tensor,         // 1×1
    neg_one_scalar: Tensor,     // 1×1
    player_1: Tensor,           // 84×1
    player_2: Tensor,           // 84×1
    forwarded_player_1: Tensor, // 63×1
    forwarded_player_2: Tensor, // 63×1
    linear_layer_1: Linear,
    linear_layer_2: Linear,
    player_weight: Tensor,
    player_bias: Tensor,
}

impl NNUEEvaluator {
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

        Ok(Self {
            player_weight,
            player_bias,
            linear_layer_1,
            linear_layer_2,
            player_1: Tensor::zeros((84,1), DType::F32, &Device::Cpu)?,
            player_2: Tensor::zeros((84,1), DType::F32, &Device::Cpu)?,
            forwarded_player_1: Tensor::zeros((63,1), DType::F32, &Device::Cpu)?,
            forwarded_player_2: Tensor::zeros((63,1), DType::F32, &Device::Cpu)?,
            zero_scalar: Tensor::zeros((1,), DType::F32, &Device::Cpu)?,
            one_scalar: Tensor::ones((1,), DType::F32, &Device::Cpu)?,
            neg_one_scalar: Tensor::from_slice(&[-1f32], (1,), &Device::Cpu)?,
            scale_scalar: Tensor::from_slice(&[16f32], (1,), &Device::Cpu)?
        })
    }

    #[allow(clippy::unused_self)]
    fn get_player_tensor(&self, player: &PlayerState) -> Tensor {
        let mut vec = Vec::with_capacity(84);

        for index in 0..QuiltBoard::TILES {
            vec.push(i32::from(player.quilt_board.get_at(index)) as f32);
        }
        vec.push(f32::from(player.get_position()));
        vec.push(f32::from(player.quilt_board.button_income));
        vec.push(player.button_balance as f32);

        Tensor::from_vec(vec, (84,), &Device::Cpu).unwrap()
    }

    pub fn initialize(&mut self, game: &Patchwork) {
        self.player_1 = self.get_player_tensor(&game.player_1);
        self.player_2 = self.get_player_tensor(&game.player_2);

        // do the forward pass for the player linear layers
        let forwarded = Tensor::stack(&[&self.player_1, &self.player_2], 0)
            .unwrap()
            .matmul(&self.player_weight.t().unwrap())
            .unwrap()
            .broadcast_add(&self.player_bias)
            .unwrap();

        self.forwarded_player_1 = forwarded.i((0, ..)).unwrap();
        self.forwarded_player_2 = forwarded.i((1, ..)).unwrap();
    }

    /// Updates the internal state of the evaluator with a new state of the game.
    ///
    /// # Arguments
    ///
    /// * `game` - The game state after the action has been executed.
    /// * `action` - The action that has been executed.
    /// * `undo_action` - Whether the action was done (false) or undone (true) in this update.
    ///
    /// # Panics
    ///
    /// Panics if the given action is invalid for the given game state.

    pub fn update_state(&mut self, game: &Patchwork, action: ActionId, was_player_1: bool) {
        if action.is_phantom() || action.is_null() {
            return;
        }

        // 1. Create Index Tensor (Which entries have changed) and
        //    Diff Tensor (What is the difference to the old value)
        // 2. MatMul the Diff Tensor by the weight matrix indexed by the Index Tensor
        // 3. Add the result to the player tensor of the current player
        if was_player_1 {
            let new_player_tensor = self.get_player_tensor(&game.player_1);
            let old_player_tensor = self.player_1.clone();
            self.player_1 = new_player_tensor.clone();

            let diff_tensor = new_player_tensor.sub(&old_player_tensor).unwrap();
            let index_tensor = diff_tensor.broadcast_gt(&self.zero_scalar).unwrap();

            // Turn One-Hot Encoding back into indices
            let mut indices = vec![];
            for index in 0..84 {
                if index_tensor.i(index).unwrap().to_scalar::<u8>().unwrap() == 1 {
                    indices.push(index as u8);
                }
            }
            let length = indices.len();
            let indices = Tensor::from_vec(indices, (length,), &Device::Cpu).unwrap();

            let player_weights = self.player_weight.index_select(&indices, 1).unwrap().t().unwrap();

            let delta_update = diff_tensor
                .index_select(&indices, 0)
                .unwrap()
                .unsqueeze(0)
                .unwrap()
                .matmul(&player_weights)
                .unwrap()
                .sum(0)
                .unwrap();

            self.forwarded_player_1 = self.forwarded_player_1.add(&delta_update).unwrap();
        } else {
            panic!("TEST THE OTHER IMPL FIRST");
        }
    }

    const fn get_special_patch_tensor(&self, game: &Patchwork) -> &Tensor {
        if matches!(
            game.turn_type,
            TurnType::SpecialPatchPlacement | TurnType::SpecialPhantom
        ) {
            &self.one_scalar
        } else {
            &self.zero_scalar
        }
    }

    const fn get_special_tile_tensor(&self, game: &Patchwork) -> &Tensor {
        if game.is_special_tile_condition_reached_by_player_1() {
            &self.one_scalar
        } else if game.is_special_tile_condition_reached_by_player_2() {
            &self.neg_one_scalar
        } else {
            &self.zero_scalar
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
}

impl NNUEEvaluator {
    pub fn test_full_feed_forward(&mut self, game: &Patchwork) -> i32 {
        self.initialize(game);

        let special_patch = self.get_special_patch_tensor(game);
        let special_tile = self.get_special_tile_tensor(game);

        // The game should already be encoded inside the ƎUИИ fields player 1 and player 2.
        let clamped_player_1 = self.forwarded_player_1.clamp(0f32, 127f32).unwrap();
        let clamped_player_2 = self.forwarded_player_2.clamp(0f32, 127f32).unwrap();

        let input_tensor /* 128×1 */ = if self.is_player_1(game) {
            Tensor::cat(&[&clamped_player_1, &clamped_player_2, &special_patch, &special_tile], 0).unwrap().unsqueeze(0).unwrap()
        }else {
            Tensor::cat(&[&clamped_player_2, &clamped_player_1, &special_patch, &special_tile], 0).unwrap().unsqueeze(0).unwrap()
        };

        let xs = self
            .linear_layer_1
            .forward(&input_tensor)
            .unwrap()
            .clamp(0f32, 127f32)
            .unwrap();

        let xs = self
            .linear_layer_2
            .forward(&xs)
            .unwrap()
            .broadcast_div(&self.scale_scalar)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .sum(0)
            .unwrap()
            .tanh()
            .unwrap();

        let eval = xs.to_scalar::<f32>().unwrap();
        if eval > 0.0 {
            (eval * evaluator_constants::POSITIVE_INFINITY as f32) as i32
        } else {
            -(eval * evaluator_constants::NEGATIVE_INFINITY as f32) as i32
        }
    }
}

impl StableEvaluator for NNUEEvaluator {}
impl Evaluator for NNUEEvaluator {
    #[rustfmt::skip]

    fn evaluate_intermediate_node(&self, game: &Patchwork) -> i32 {
        let special_patch = self.get_special_patch_tensor(game);
        let special_tile = self.get_special_tile_tensor(game);

        // The game should already be encoded inside the ƎUИИ fields player 1 and player 2.
        let clamped_player_1 = self.forwarded_player_1.clamp(0f32, 127f32).unwrap();
        let clamped_player_2 = self.forwarded_player_2.clamp(0f32, 127f32).unwrap();

        let input_tensor /* 128×1 */ = if self.is_player_1(game) {
            Tensor::cat(&[&clamped_player_1, &clamped_player_2, &special_patch, &special_tile], 0).unwrap().unsqueeze(0).unwrap()
        }else {
            Tensor::cat(&[&clamped_player_2, &clamped_player_1, &special_patch, &special_tile], 0).unwrap().unsqueeze(0).unwrap()
        };

        let xs = self.linear_layer_1
            .forward(&input_tensor)
            .unwrap()
            .clamp(0f32, 127f32)
            .unwrap();

        let xs = self.linear_layer_2
            .forward(&xs)
            .unwrap()
            .broadcast_div(&self.scale_scalar)
            .unwrap()
            .squeeze(0)
            .unwrap()
            .sum(0)
            .unwrap()
            .tanh()
            .unwrap();

        let eval = xs.to_scalar::<f32>().unwrap();
        if eval > 0.0 {
            (eval * evaluator_constants::POSITIVE_INFINITY as f32 ) as i32
        } else {
            -(eval * evaluator_constants::NEGATIVE_INFINITY as f32 ) as i32
        }
    }
}

// ```text
//                          ⬐ σ=ReLU
//         ┌─       ◯ ─── ┌──┐-_
//         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ⎺⎻⎼⎽_  ⬐ σ=ReLU
// Current │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ ┌──┐-
// Player  │  Pos   ◯ ─── │ C│ ??× ⋮ ⋰  ⋱ │  │- ⟍
// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ‾⟍ ╲  ⬐ σ=ReLU
//         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋰⋱  ┌──┐
//         ┌─       ◯ ─── ┌──┐-_     __── │  │ ⋱⋰  │  │ ⟍
//         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ──── │  │ ⋰⋱  │  │ ⟍ ⟍
// Other   │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ │  │ ⋱⋰  │  │ ‾-_⟍ ⟍
// Player  │  Pos   ◯ ─── │ C│ ??× ⋮ ⋰  ⋱ │F │ ⋰⋱  │F │ ────── ◯ ─ σ(tanh) ⭢ Evaluation
// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │ C│ ??× │ C│ ??× _─⎺
//         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋰⋱  │  │ _--‾ ╱
//         ┌─ 1     ◯ ‾-_ ┌──┐-_     __── │  │ ⋱⋰  │  │ ⟋⟋ ⟋
//         │  2     ◯ ─── │EM│  ‾─ ◯ ──── │  │ ⋰⋱  │  │⟋ ⟋
// Patches │  3     ◯ ─── │BE│ 18× ⋮ ⋱  ⋰ │  │ ⋱⋰  └──┘‾
// (6)     │  4     ◯ ─── │DD│ 6 ─ ⋮ ⋰  ⋱ │  │ _⟋ ╱
//         │  5     ◯ ─── │IN│ _⎽- ◯ ──── │  │- ⟋
//         └─ 6     ◯ _-‾ └──┘‾      ‾‾─_ └──┘‾
// Flags   ┌─ SP    ◯ ──────────── ◯ _-‾ ╱
// (2)     └─ ST    ◯ ──────────── ◯ _-‾
// ```
//
// ```text
//                  ┌──── 16 bit ───┐ ┌──────── 8 bit ─────────┐
//                          ⬐ σ=ReLU
//         ┌─       ◯ ─── ┌──┐-_
//         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ⎺⎻⎼⎽_  ⬐ σ=ReLU
// Current │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ ┌──┐-_
// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │-_‾─_    ⬐ σ=ReLU
// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋰⋱ ‾┌──┐_
//         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │ ⋱⋰  │  │-_‾‾─_
//         ┌─       ◯ ─── ┌──┐-_     __── │F │ ⋰⋱  │F │ ??×  ⟍
//         │  81×QB ⋮⋮ ─── │  │  ‾─ ◯ ──── │ C│ ??× │ C│ ────── ◯ ─ σ(tanh) ⭢ Evaluation
// Other   │        ◯ ─── │F │ ─── ⋮ ⋱  ⋰ │  │ ⋱⋰  │  │⎼⎻⎺_─‾ ⟋
// Player  │  Pos   ◯ ─── │ C│ 63× ⋮ ⋰  ⋱ │  │ ⋰⋱  │  │─‾__─‾
// (84)    │  BI    ◯ ─── │  │ _⎽- ◯ ──── │  │ ⋰⋱ _└──┘‾
//         └─ BB    ◯ ─── └──┘‾      ‾‾── │  │-‾_─‾
// Flags   ┌─ SP    ◯ ──────────── ◯ ──‾_ └──┘-‾
// (2)     └─ ST    ◯ ──────────── ◯ ⎼⎻⎺
//         2×84+2=170        2×63+2=128    32
//       input features      parameters1  32
// ```
