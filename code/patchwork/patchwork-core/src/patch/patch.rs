use std::fmt::{Display, Error, Formatter};

use crate::PatchManager;

/// Represents a patch in the game Patchwork.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Patch {
    /// The unique ID of the patch.
    pub id: u8,
    /// The amount of buttons that the patch costs.
    pub button_cost: u8,
    /// The amount of time that the patch costs.
    pub time_cost: u8,
    /// The amount of buttons you get as additional income from this patch.
    pub button_income: u8,
}

impl Patch {
    /// Returns the amount of tiles that this patch has.
    #[inline]
    #[must_use]
    pub fn amount_tiles(&self) -> u32 {
        PatchManager::get_transformations(self.id)[0].tiles.count_ones()
    }
}

impl Display for Patch {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let tiles = PatchManager::get_tiles(self.id);
        for row in tiles {
            for tile in row {
                if *tile == 1 {
                    write!(f, "█")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }

        writeln!(f, "Id: {}", self.id)?;
        writeln!(f, "Income: {}", self.button_income)?;
        writeln!(f, "Button cost: {}", self.button_cost)?;
        write!(f, "Time cost: {}", self.time_cost)
    }
}
