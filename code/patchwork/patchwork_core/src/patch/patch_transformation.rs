use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PatchTransformation {
    /// The row of the patch in the patch board.
    pub row: usize,
    /// The column of the patch in the patch board.
    pub column: usize,
    /// The transformation of the patch (rotation and orientation).
    pub transformation: u8,
    /// The tiles of the patch.
    pub tiles: u128,
}

impl PatchTransformation {
    pub const ROTATION_0: u8 = 0b000;
    pub const ROTATION_90: u8 = 0b001;
    pub const ROTATION_180: u8 = 0b010;
    pub const ROTATION_270: u8 = 0b011;
    pub const FLIPPED: u8 = 0b100;
    pub const FLIPPED_ROTATION_90: u8 = 0b101;
    pub const FLIPPED_ROTATION_180: u8 = 0b110;
    pub const FLIPPED_ROTATION_270: u8 = 0b111;
}

impl Debug for PatchTransformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("PatchTransformation")
            .field("row", &self.row)
            .field("column", &self.column)
            .field("rotation", &self.rotation())
            .field("flipped", &self.flipped())
            .field("tiles", &format_args!("{:#083b}", self.tiles))
            .finish()
    }
}

impl PatchTransformation {
    /// Returns the rotation as 0, 90, 180 or 270 degree rotation.
    #[inline]
    pub fn rotation(&self) -> u32 {
        match self.transformation {
            PatchTransformation::ROTATION_0 | PatchTransformation::FLIPPED => 0,
            PatchTransformation::ROTATION_90 | PatchTransformation::FLIPPED_ROTATION_90 => 90,
            PatchTransformation::ROTATION_180 | PatchTransformation::FLIPPED_ROTATION_180 => 180,
            PatchTransformation::ROTATION_270 | PatchTransformation::FLIPPED_ROTATION_270 => 270,
            _ => panic!("[PatchTransformation][rotation] Invalid transformation!"),
        }
    }

    /// Returns whether the patch is flipped.
    #[inline]
    pub fn flipped(&self) -> bool {
        self.transformation & 0b100 != 0
    }

    /// Returns the rotation as 0 (0°), 1 (90°), 2 (180°) or 3 (270°).
    #[inline]
    pub fn rotation_flag(&self) -> u8 {
        self.transformation & 0b011
    }

    /// Returns the orientation as 0 (normal) or 1 (flipped).
    #[inline]
    pub fn orientation_flag(&self) -> u8 {
        (self.transformation & 0b100) >> 2
    }

    // pub fn get_row_and_column_of_tiles(tiles: u128) -> (u32, u32) {
    //     let mut min_row = 0;
    //     let mut min_column = 0;

    //     for row in 0..QuiltBoard::ROWS {
    //         for column in 0..QuiltBoard::COLUMNS {
    //             let index = row * QuiltBoard::COLUMNS + column;
    //             if (tiles >> index) & 1 > 0 {
    //                 min_row = row.min(min_row);
    //                 min_column = column.min(min_column);
    //             }
    //         }

    //         if min_row != 0 && min_row + 4 < row {
    //             break;
    //         }
    //     }

    //     (min_row, min_column)
    // }
}