use std::fs::OpenOptions;
use std::io::{Result, Write};

use action_orderer::{PATCH_PLACEMENT_ENDGAME_TABLE, PATCH_PLACEMENT_OPENING_TABLE};
use patchwork_core::{PatchManager, QuiltBoard};

fn main() {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("values.csv")
        .unwrap();
    drop(file);

    for patch_id in 0..PatchManager::AMOUNT_OF_NORMAL_PATCHES as usize {
        let to_console =
            patch_id == 17 || patch_id == 20 || patch_id == 21 || patch_id == 22 || patch_id == 23 || patch_id == 24;

        do_single_patch(patch_id as u8, to_console).unwrap();
    }
}

#[allow(clippy::needless_range_loop)]
fn do_single_patch(patch_id: u8, output_to_console: bool) -> Result<()> {
    let mut opening = [[0.0; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize];
    let mut endgame = [[0.0; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize];

    for (index, transformation) in PatchManager::get_transformations(patch_id).iter().enumerate() {
        let board = QuiltBoard::from_bits(transformation.tiles);

        for row in 0..QuiltBoard::ROWS as usize {
            for column in 0..QuiltBoard::COLUMNS as usize {
                if board.get(row as u8, column as u8) {
                    opening[row][column] += PATCH_PLACEMENT_OPENING_TABLE[patch_id as usize][index];
                    endgame[row][column] += PATCH_PLACEMENT_ENDGAME_TABLE[patch_id as usize][index];
                }
            }
        }
    }

    if output_to_console {
        println!("# Opening Patch {patch_id:?}");
        println!("patch_{patch_id}_op = {opening:?}");
        println!("# Endgame Patch {patch_id:?}");
        println!("patch_{patch_id}_end = {endgame:?}");
        println!();
    }

    let opening = normalize(opening);
    let endgame = normalize(endgame);

    let mut file = OpenOptions::new().append(true).create(true).open("values.csv")?;

    writeln!(file, "Opening Patch {patch_id:?}")?;
    for row in 0..QuiltBoard::ROWS as usize {
        writeln!(
            file,
            "{}",
            opening[row]
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(";")
        )?;
    }
    writeln!(file, "Endgame Patch {patch_id:?}")?;
    for row in 0..QuiltBoard::ROWS as usize {
        writeln!(
            file,
            "{}",
            endgame[row]
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(";")
        )?;
    }
    writeln!(file)?;

    Ok(())
}

#[allow(clippy::needless_range_loop)]
fn normalize(
    mut board: [[f64; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize],
) -> [[f64; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize] {
    let mut exp_board = [[0.0; QuiltBoard::COLUMNS as usize]; QuiltBoard::ROWS as usize];

    let mut sum = 0.0;

    for row in 0..QuiltBoard::ROWS as usize {
        for column in 0..QuiltBoard::COLUMNS as usize {
            let value = board[row][column];
            exp_board[row][column] = value.exp();
            sum += exp_board[row][column];
        }
    }

    for row in 0..QuiltBoard::ROWS as usize {
        for column in 0..QuiltBoard::COLUMNS as usize {
            board[row][column] = exp_board[row][column] / sum;
        }
    }

    board
}
