use std::fmt;

use serde::ser::{SerializeSeq, SerializeStruct};

use patchwork_lib::{time_board_flags, Patch, PatchManager, Patchwork, PlayerState, QuiltBoard, TimeBoard};

pub struct PatchworkState(pub Patchwork);

impl fmt::Debug for PatchworkState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Clone for PatchworkState {
    fn clone(&self) -> Self {
        PatchworkState(self.0.clone())
    }
}

impl PartialEq for PatchworkState {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for PatchworkState {}

impl serde::Serialize for PatchworkState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let PatchworkState(state) = self;
        let mut serialized_state = serializer.serialize_struct("Patchwork", 8)?;

        serialized_state.serialize_field(
            "patches",
            &PatchesSerialization {
                patches: &state.patches,
            },
        )?;
        serialized_state.serialize_field(
            "time_board",
            &TimeBoardSerialization {
                time_board: &state.time_board,
            },
        )?;
        serialized_state.serialize_field(
            "player_1",
            &PlayerSerialization {
                player: &state.player_1,
            },
        )?;
        serialized_state.serialize_field(
            "player_2",
            &PlayerSerialization {
                player: &state.player_2,
            },
        )?;
        serialized_state.serialize_field("turn_type", &state.turn_type)?;
        serialized_state.serialize_field("status_flags", &StatusFlagSerialization { state })?;
        serialized_state.serialize_field("notation", &state.save_to_notation_with_phantom_state(true).unwrap())?;
        serialized_state.end()
    }
}

struct TimeBoardSerialization<'a> {
    time_board: &'a TimeBoard,
}

impl serde::Serialize for TimeBoardSerialization<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        struct TimeBoardTileSerialization {
            player_1: bool,
            player_2: bool,
            special_patch: bool,
            button_income_trigger: bool,
        }

        struct TimeBoardBoardSerialization<'a> {
            time_board: &'a TimeBoard,
        }

        struct TimeBoardSpecialPatches<'a> {
            time_board: &'a TimeBoard,
        }

        struct TimeBoardButtonIncomeTriggers<'a> {
            time_board: &'a TimeBoard,
        }

        impl serde::Serialize for TimeBoardTileSerialization {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut serialized_tile = serializer.serialize_struct("TimeBoardTile", 4)?;
                serialized_tile.serialize_field("player_1", &self.player_1)?;
                serialized_tile.serialize_field("player_2", &self.player_2)?;
                serialized_tile.serialize_field("special_patch", &self.special_patch)?;
                serialized_tile.serialize_field("button_income_trigger", &self.button_income_trigger)?;
                serialized_tile.end()
            }
        }
        impl serde::Serialize for TimeBoardBoardSerialization<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut serialized_board = serializer.serialize_seq(Some(self.time_board.tiles.len()))?;
                for tile in &self.time_board.tiles {
                    serialized_board.serialize_element(&TimeBoardTileSerialization {
                        player_1: tile & time_board_flags::PLAYER_1 != 0,
                        player_2: tile & time_board_flags::PLAYER_2 != 0,
                        special_patch: tile & time_board_flags::SPECIAL_PATCH != 0,
                        button_income_trigger: tile & time_board_flags::BUTTON_INCOME_TRIGGER != 0,
                    })?;
                }
                serialized_board.end()
            }
        }

        impl serde::Serialize for TimeBoardSpecialPatches<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut serialized_special_patches = serializer.serialize_seq(None)?;
                for tile in &self.time_board.tiles {
                    if tile & time_board_flags::SPECIAL_PATCH != 0 {
                        serialized_special_patches.serialize_element(&tile)?;
                    }
                }
                serialized_special_patches.end()
            }
        }

        impl serde::Serialize for TimeBoardButtonIncomeTriggers<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut serialized_button_income_triggers = serializer.serialize_seq(None)?;
                for tile in &self.time_board.tiles {
                    if tile & time_board_flags::BUTTON_INCOME_TRIGGER != 0 {
                        serialized_button_income_triggers.serialize_element(&tile)?;
                    }
                }
                serialized_button_income_triggers.end()
            }
        }

        let mut serialized_time_board = serializer.serialize_struct("TimeBoard", 3)?;
        serialized_time_board.serialize_field(
            "player_1",
            &self.time_board.get_player_position(time_board_flags::PLAYER_1),
        )?;
        serialized_time_board.serialize_field(
            "player_2",
            &self.time_board.get_player_position(time_board_flags::PLAYER_2),
        )?;
        serialized_time_board.serialize_field(
            "special_patches",
            &TimeBoardSpecialPatches {
                time_board: self.time_board,
            },
        )?;
        serialized_time_board.serialize_field(
            "button_income_triggers",
            &TimeBoardButtonIncomeTriggers {
                time_board: self.time_board,
            },
        )?;
        serialized_time_board.serialize_field(
            "board",
            &TimeBoardBoardSerialization {
                time_board: self.time_board,
            },
        )?;
        serialized_time_board.end()
    }
}

struct PatchSerialization {
    patch: &'static Patch,
}

impl serde::Serialize for PatchSerialization {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialized_patch = serializer.serialize_struct("Patch", 5)?;
        serialized_patch.serialize_field("id", &self.patch.id)?;
        serialized_patch.serialize_field("button_cost", &self.patch.button_cost)?;
        serialized_patch.serialize_field("time_cost", &self.patch.time_cost)?;
        serialized_patch.serialize_field("button_income", &self.patch.button_income)?;
        serialized_patch.serialize_field("tiles", PatchManager::get_tiles(self.patch.id))?;
        serialized_patch.end()
    }
}

struct PatchesSerialization<'a> {
    patches: &'a Vec<&'static Patch>,
}

impl serde::Serialize for PatchesSerialization<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialized_patches = serializer.serialize_seq(Some(self.patches.len()))?;
        for patch in self.patches {
            serialized_patches.serialize_element(&PatchSerialization { patch })?;
        }
        serialized_patches.end()
    }
}

struct QuiltBoardSerialization<'a> {
    quilt_board: &'a QuiltBoard,
}

impl serde::Serialize for QuiltBoardSerialization<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialized_quilt_board = serializer.serialize_struct("QuiltBoard", 2)?;
        let mut board = Vec::with_capacity(QuiltBoard::ROWS as usize);
        serialized_quilt_board.serialize_field("button_income", &self.quilt_board.button_income)?;
        serialized_quilt_board.serialize_field("tiles", {
            for row in 0..QuiltBoard::ROWS {
                let mut row_vec = Vec::with_capacity(QuiltBoard::COLUMNS as usize);
                for col in 0..QuiltBoard::COLUMNS {
                    row_vec.push(self.quilt_board.get(row, col));
                }
                board.push(row_vec);
            }
            &board
        })?;
        serialized_quilt_board.end()
    }
}

struct PlayerSerialization<'a> {
    player: &'a PlayerState,
}

impl serde::Serialize for PlayerSerialization<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialized_player = serializer.serialize_struct("Player", 3)?;
        serialized_player.serialize_field("position", &self.player.get_position())?;
        serialized_player.serialize_field("button_balance", &self.player.button_balance)?;
        serialized_player.serialize_field("quilt_board", {
            &QuiltBoardSerialization {
                quilt_board: &self.player.quilt_board,
            }
        })?;
        serialized_player.end()
    }
}

struct StatusFlagSerialization<'a> {
    state: &'a Patchwork,
}

impl serde::Serialize for StatusFlagSerialization<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serialized_status_flags = serializer.serialize_struct("StatusFlags", 3)?;
        serialized_status_flags.serialize_field("current_player", if self.state.is_player_1() { &1 } else { &2 })?;
        serialized_status_flags.serialize_field(
            "special_tile",
            if self.state.is_special_tile_condition_reached_by_player_1() {
                &Some(1)
            } else if self.state.is_special_tile_condition_reached_by_player_2() {
                &Some(2)
            } else {
                &None
            },
        )?;
        serialized_status_flags.serialize_field(
            "first_goal",
            if self.state.player_1_was_first_to_reach_goal() {
                &Some(1)
            } else if self.state.player_2_was_first_to_reach_goal() {
                &Some(2)
            } else {
                &None
            },
        )?;
        serialized_status_flags.end()
    }
}
