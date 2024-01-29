use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{Embedding, LSTMConfig, VarBuilder, LSTM, RNN};
use patchwork_core::{Patch, PatchManager, Patchwork, QuiltBoard, TimeBoard};

pub struct GameEncoder<'a> {
    device: &'a Device,
    patch_embeddings: Embedding,
    patch_lstm: LSTM,
}

impl<'a> GameEncoder<'a> {
    const NORMAL_PATCH_LAYERS: usize = 3;

    pub fn new(vb: VarBuilder, device: &Device) -> Result<Self> {
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

        Ok(Self {
            patch_embeddings,
            patch_lstm,
            device,
        })
    }

    #[rustfmt::skip]
    pub fn encode_state(&self, game: &Patchwork) -> Result<Tensor> {
        let patches = self.encode_patches(&game.patches)?;
        let player_1_quilt_board = self.encode_quilt_board(&game.player_1.quilt_board)?;
        let player_2_quilt_board = self.encode_quilt_board(&game.player_2.quilt_board)?;
        let current_player = self.encode_current_player(game)?;
        let time_board = self.encode_time_board(&game.time_board)?;

        Tensor::cat(&[&player_1_quilt_board, &player_2_quilt_board, &current_player, &patches, &time_board], 0)
    }

    #[rustfmt::skip]
    fn encode_patches(&self, patches: &Vec<&'static Patch>) -> Result<Tensor> {
        let beginning_patch_ids = Tensor::from_iter(patches.iter().take(Self::NORMAL_PATCH_LAYERS).map(|patch| patch.id), self.device)?;
        let beginning_patches = self.patch_embeddings.forward(&beginning_patch_ids)?;

        let padding = Tensor::zeros(&[patches.len().max(Self::NORMAL_PATCH_LAYERS), 81], DType::F32, self.device)?;

        let ending_patches = if patches.len() <= Self::NORMAL_PATCH_LAYERS {
            &Tensor::zeros(&[0, 81], DType::F32, self.device)?
        } else {
            let ending_patch_ids = Tensor::from_iter(patches.iter().skip(Self::NORMAL_PATCH_LAYERS).map(|patch| patch.id), self.device)?;
            let ending_patches = self.patch_embeddings.forward(&ending_patch_ids)?.reshape((
                1,
                ending_patch_ids.dim(0)?,
                QuiltBoard::TILES as usize,
            ))?;

            self.patch_lstm.seq(&ending_patches)?[0].h()
        };

        let patches = Tensor::cat(&[&beginning_patches, &padding, ending_patches], 0)?.reshape((
            patches.len(),
            QuiltBoard::ROWS as usize,
            QuiltBoard::COLUMNS as usize,
        ))?; /* (0..33) → (0.33)×81 → (0..33)×9×9 */

        Ok(patches)
    }

    fn encode_quilt_board(&self, quilt_board: &QuiltBoard) -> Result<Tensor> {
        let mut quilt_board_slice = [0.0; QuiltBoard::COLUMNS as usize * QuiltBoard::ROWS as usize];
        for index in 0..QuiltBoard::TILES {
            quilt_board_slice[index as usize] = quilt_board.get_at(index) as u8 as f32;
        }

        Tensor::from_slice(
            &quilt_board_slice,
            (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
            self.device,
        )
    }

    fn encode_current_player(&self, game: &Patchwork) -> Result<Tensor> {
        if game.is_player_1() {
            Tensor::ones(
                (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
                DType::F32,
                self.device,
            )
        } else {
            Tensor::zeros(
                (QuiltBoard::ROWS as usize, QuiltBoard::COLUMNS as usize),
                DType::F32,
                self.device,
            )
        }
    }

    fn encode_time_board(&self, time_board: &TimeBoard) -> Result<Tensor> {
        // fully connected layer inputs -> hidden -> out 81 -> reshape 9x9
        todo!()
    }
}

// Encodes the given game state of patchwork into a 9x9xN tensor so that
// it can be used as input for the alphazero neural network.
//
// The tensor consists of the following layers
// * 9x9 layer for the current quilt board of player 1
//   * 0 for empty squares
//   * 1 for squares covered by a patch
// * 9x9 layer for the current quilt board of player 2
//   * 0 for empty squares
//   * 1 for squares covered by a patch
// * 9x9 uniform layer who's turn it is
//   * 0 for player 1
//   * 1 for player 2
// * 9x9 layer for the time board
//
//
//
// TODO:
// * embedding layer
// * patches (first 3 at least)
// * position of player 1 and 2 on time board
// * distance from each other
//
// * (special patch locations)
// * (button income trigger locations)
// * (the player who was first to reach the goal)
//
//     /// struct EncodedState {
//     /// The patches encoded into one tensor
//     /// First there are 3 patches (starting from the beginning) that are
//     /// directly out of an embedding layer
//     /// All the following patches are encoded through an LSTM layer into the
//     /// last layer of the tensor
//     patches: Tensor, /* 4×9×9 */
//     /// The quilt board of player 1
//     /// * 0 for empty squares
//     /// * 1 for squares covered by a patch
//     player_1_quilt_board: Tensor, /* 9×9 */
//     /// The quilt board of player 2
//     /// * 0 for empty squares
//     /// * 1 for squares covered by a patch
//     player_2_quilt_board: Tensor, /* 9×9 */
//     /// The current player
//     /// * 0 for player 1
//     /// * 1 for player 2
//     current_player: Tensor, /* 9×9 */
//     /// The time board
//     ///
//     time_board: Tensor, /* 9×9 */
// }
