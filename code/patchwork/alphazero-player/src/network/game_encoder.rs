use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{Embedding, LSTMConfig, Linear, VarBuilder, LSTM, RNN};
use patchwork_core::{time_board_flags, Patch, PatchManager, Patchwork, QuiltBoard, TimeBoard};

/// A game encoder encodes a game of patchwork into a tensor.
///
/// `PATCH_LAYERS` is the amount of patches that are encoded into a separate
/// layer. All other patches will be encoded into a single layer via an lstm.
#[derive(Debug, Clone)]
pub struct GameEncoder<const PATCH_LAYERS: usize> {
    device: Device,
    patch_embeddings: Embedding,
    patch_lstm: LSTM,
    time_board_fc1: Linear,
    time_board_fc2: Linear,
}

impl<const PATCH_LAYERS: usize> GameEncoder<PATCH_LAYERS> {
    /// The amount of input features for the time board neural network.
    const TIME_BOARD_INPUT_SIZE: usize = 3
        + TimeBoard::MAX_POSITION as usize
        + 1
        + TimeBoard::AMOUNT_OF_SPECIAL_PATCHES
        + TimeBoard::AMOUNT_OF_BUTTON_INCOME_TRIGGERS;
    /// The amount of hidden features for the time board neural network.
    const TIME_BOARD_HIDDEN_LAYER_SIZE: usize = (Self::TIME_BOARD_INPUT_SIZE + QuiltBoard::TILES as usize) / 2;

    /// Creates a new game encoder.
    ///
    /// # Arguments
    ///
    /// * `vb` - The variable builder to use for creating the variables.
    /// * `device` - The device to use for the variables.
    ///
    /// # Returns
    ///
    /// A new game encoder.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(vb: VarBuilder<'_>, device: Device) -> Result<Self> {
        let patch_embeddings = candle_nn::embedding(
            PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize,
            QuiltBoard::TILES as usize,
            vb.pp("patch_embeddings"),
        )?;

        let patch_lstm = candle_nn::lstm(
            QuiltBoard::TILES as usize,
            QuiltBoard::TILES as usize,
            LSTMConfig::default(),
            vb.pp("patch_lstm"),
        )?;

        let time_board_fc1 = candle_nn::linear(
            Self::TIME_BOARD_INPUT_SIZE,
            Self::TIME_BOARD_HIDDEN_LAYER_SIZE,
            vb.pp("time_board_fc1"),
        )?;
        let time_board_fc2 = candle_nn::linear(
            Self::TIME_BOARD_HIDDEN_LAYER_SIZE,
            QuiltBoard::TILES as usize,
            vb.pp("time_board_fc2"),
        )?;

        Ok(Self {
            device,
            patch_embeddings,
            patch_lstm,
            time_board_fc1,
            time_board_fc2,
        })
    }

    /// Encodes the given games into a tensor of shape (`batch_size`, `PATCH_LAYERS` + 5, 9, 9)
    /// The tensor contains the encoded patches, the quilt boards, the current
    /// player and the time board.
    ///
    /// # Arguments
    ///
    /// * `games` - The games to encode of length `batch_size`.
    ///
    /// # Returns
    ///
    /// A tensor of shape (`batch_size`, `PATCH_LAYERS` + 5, 9, 9) containing the encoded game.
    #[rustfmt::skip]
    pub fn encode_state(&self, games: &[&Patchwork]) -> Result<Tensor> {
        let encoded_games = games.iter().map(|game| {
            let patches = self.encode_patches(&game.patches)?;                               // PATCH_LAYERS + 1
            let player_1_quilt_board = self.encode_quilt_board(&game.player_1.quilt_board)?; // 1 layer
            let player_2_quilt_board = self.encode_quilt_board(&game.player_2.quilt_board)?; // 1 layer
            let current_player = self.encode_current_player(game)?;                          // 1 layer
            let time_board = self.encode_time_board(&game.time_board)?;                      // 1 layer

            Tensor::cat(&[&player_1_quilt_board, &player_2_quilt_board, &current_player, &patches, &time_board], 0)
        }).collect::<Result<Vec<_>>>()?;

        Tensor::stack(&encoded_games, 0)
    }

    /// Encodes the given patches into a tensor.
    ///
    /// The tensor is of shape ([`PATCH_LAYERS`] + 1, 9, 9) and
    /// contains the encoded patches.
    ///
    /// The first [`PATCH_LAYERS`] layers contain the first
    /// [`PATCH_LAYERS`] patches. The last layer contains the
    /// remaining patches.
    ///
    /// The last layer contains the remaining patches. The is done by
    /// many-to-one encoding with a LSTM starting with the last patch and ending
    /// with the first patch.
    ///
    /// If there are less than [`PATCH_LAYERS`] patches left the
    /// other layers are padded with zeros.
    ///
    /// # Returns
    ///
    /// A tensor of shape ([`PATCH_LAYERS`] + 1, 9, 9) containing
    /// the encoded patches.
    #[rustfmt::skip]
    fn encode_patches(&self, patches: &[&'static Patch]) -> Result<Tensor> {
        let beginning_patch_ids = Tensor::from_iter(patches.iter().take(PATCH_LAYERS).map(|patch| patch.id), &self.device)?;
        let beginning_patches = self.patch_embeddings.forward(&beginning_patch_ids)?.t()?;

        let padding = Tensor::zeros(&[QuiltBoard::TILES as usize, PATCH_LAYERS - patches.len().clamp(0, PATCH_LAYERS)], DType::F32, &self.device)?;

        if patches.len() <= PATCH_LAYERS {
            return Tensor::cat(&[&beginning_patches, &padding,  &Tensor::zeros(&[1, 81], DType::F32, &self.device)?], 1)?.reshape((
                PATCH_LAYERS + 1,
                QuiltBoard::ROWS as usize,
                QuiltBoard::COLUMNS as usize,
            ));
        }

        let ending_patch_ids = Tensor::from_iter(patches.iter().skip(PATCH_LAYERS).map(|patch| patch.id), &self.device)?;
        let ending_patches = self.patch_embeddings.forward(&ending_patch_ids)?;

        let mut lstm_state = self.patch_lstm.zero_state(1)?;
        for index in (0..ending_patches.dim(0)?).rev() {
            let patch = ending_patches.get(index)?.unsqueeze(0)?;
            lstm_state = self.patch_lstm.step(&patch, &lstm_state)?;
        }
        let ending_patches = lstm_state.h().t()?;

        let patches = Tensor::cat(&[&beginning_patches, &padding, &ending_patches], 1)?.reshape((
            PATCH_LAYERS + 1,
            QuiltBoard::ROWS as usize,
            QuiltBoard::COLUMNS as usize,
        ))?;

        Ok(patches)
    }

    /// Encodes the given quilt board into a 9x9 tensor.
    /// The tensor is filled with 0s for empty squares and 1s for squares
    /// covered by a patch.
    ///
    /// # Returns
    ///
    /// A tensor of shape (9, 9) filled with 0s for empty squares and 1s for
    /// squares covered by a patch.
    fn encode_quilt_board(&self, quilt_board: &QuiltBoard) -> Result<Tensor> {
        let mut quilt_board_slice = [0.0; QuiltBoard::COLUMNS as usize * QuiltBoard::ROWS as usize];
        for index in 0..QuiltBoard::TILES {
            quilt_board_slice[index as usize] = f32::from(u8::from(quilt_board.get_at(index)));
        }

        Tensor::from_slice(
            &quilt_board_slice,
            (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
            &self.device,
        )?
        .unsqueeze(0)
    }

    /// Encodes the current player into a 9x9 tensor.
    ///
    /// The tensor is filled with 1s if it is player 1's turn and 0s if it is
    /// player 2's turn.
    ///
    /// # Returns
    ///
    /// A tensor of shape (9, 9) filled with 1s if it is player 1's turn and 0s
    /// if it is player 2's turn.
    fn encode_current_player(&self, game: &Patchwork) -> Result<Tensor> {
        if game.is_player_1() {
            Tensor::ones(
                (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
                DType::F32,
                &self.device,
            )?
            .unsqueeze(0)
        } else {
            Tensor::zeros(
                (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
                DType::F32,
                &self.device,
            )?
            .unsqueeze(0)
        }
    }

    /// Encodes the time board into a 9x9 tensor.
    ///
    /// For this first different values are normalized to [0, 1] and then
    /// passed through two linear layers with relu activation. The resulting
    /// tensor is then reshaped to (9, 9).
    ///
    /// The different values are:
    /// * player 1 position [0, `TimeBoard::MAX_POSITION`] normalized to [0, 1]
    /// * player 2 position [0, `TimeBoard::MAX_POSITION`] normalized to [0, 1]
    /// * player distance [-6, 6] normalized to [-1, 1]
    /// * The position of all button income triggers (9) normalized to (0, 1]
    /// * The position of all special patches (5) normalized to (0, 1)
    /// * Each tile of the time board (54) normalized to [0, 1)
    ///
    /// # Returns
    ///
    /// A tensor of shape (9, 9) containing the encoded time board.
    ///
    /// # Panics
    ///
    /// Panics if the time board input has the wrong length. This should never
    /// happen.
    fn encode_time_board(&self, time_board: &TimeBoard) -> Result<Tensor> {
        let mut time_board_input = Vec::with_capacity(QuiltBoard::TILES as usize);
        let (player_1_pos, player_2_pos) = time_board.get_player_positions();

        // player 1 position [0, TimeBoard::MAX_POSITION] normalized to [0, 1]
        time_board_input.push(f32::from(player_1_pos) / f32::from(TimeBoard::MAX_POSITION));
        // player 2 position [0, TimeBoard::MAX_POSITION] normalized to [0, 1]
        time_board_input.push(f32::from(player_2_pos) / f32::from(TimeBoard::MAX_POSITION));
        // player distance [-6, 6] normalized to [-1, 1]
        time_board_input.push((f32::from(player_1_pos) - f32::from(player_2_pos)) / 6.0);
        // The position of all button income triggers (9) normalized to (0, 1]
        for index in time_board.get_button_income_triggers() {
            time_board_input.push(f32::from(*index) / f32::from(TimeBoard::MAX_POSITION));
        }
        // The position of all special patches (5) normalized to (0, 1)
        for index in time_board.get_special_patches() {
            time_board_input.push(f32::from(*index) / f32::from(TimeBoard::MAX_POSITION));
        }
        // Each tile of the time board (54) normalized to [0, 1)
        time_board_input.extend(
            time_board
                .tiles
                .iter()
                .map(|tile| f32::from(*tile) / f32::from(time_board_flags::MAX_VALUE)),
        );

        debug_assert_eq!(
            time_board_input.len(),
            Self::TIME_BOARD_INPUT_SIZE,
            "[GameEncoder::encode_time_board] Time board input has wrong length {}, should be {}",
            time_board_input.len(),
            Self::TIME_BOARD_INPUT_SIZE
        );

        let mut xs = Tensor::from_vec(time_board_input, (1, Self::TIME_BOARD_INPUT_SIZE), &self.device)?;
        xs = self.time_board_fc1.forward(&xs)?.relu()?;
        xs = self.time_board_fc2.forward(&xs)?.relu()?;
        xs.reshape((1, QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize))
    }
}
