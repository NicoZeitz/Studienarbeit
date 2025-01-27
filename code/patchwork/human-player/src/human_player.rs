use std::{
    collections::HashSet,
    io::{self, Write},
};

use patchwork_core::{ActionId, PatchManager, PatchTransformation, Patchwork, Player, PlayerResult, QuiltBoard};
use rand::Rng;
use regex::Regex;

/// A player that is human
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HumanPlayer {
    /// The name of the player.
    name: String,
}

impl HumanPlayer {
    /// Creates a new [`HumanPlayer`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for HumanPlayer {
    fn default() -> Self {
        Self::new("Human Player".to_string())
    }
}

impl Player for HumanPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> PlayerResult<ActionId> {
        let valid_actions = game.get_valid_actions();

        if valid_actions[0].is_special_patch_placement() {
            self.handle_special_patch_action(valid_actions)
        } else {
            self.handle_normal_action(game, valid_actions)
        }
    }
}

impl HumanPlayer {
    // Gets the input for the special patch placement.
    //
    // # Arguments
    //
    // * `state` - The current state.
    // * `valid_actions` - The valid actions.
    //
    // # Returns
    //
    // The action.
    fn handle_special_patch_action(&mut self, valid_actions: Vec<ActionId>) -> PlayerResult<ActionId> {
        let mut valid_actions = valid_actions;
        let initial_prompt = format!(
            "Player '{}' has to place the special patch. Please enter the row and column of the patch (row, column):",
            self.name
        );
        #[allow(clippy::redundant_clone)] // This clone is needed but clippy does not get this
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt)?;

            if human_input == "skip" {
                let index = rand::thread_rng().gen_range(0..valid_actions.len());
                return Ok(valid_actions.remove(index));
            }

            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'row, column'. {initial_prompt}");
                continue;
            }

            let optional_row = human_inputs[0].parse::<u8>();
            let optional_column = human_inputs[1].parse::<u8>();

            if optional_row.is_err() || optional_column.is_err() {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let row = optional_row.unwrap();
            let column = optional_column.unwrap();

            if row > QuiltBoard::ROWS || column > QuiltBoard::COLUMNS {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let patch_position = (row - 1, column - 1);

            for action in &valid_actions {
                let action_row = action.get_row();
                let action_column = action.get_column();

                if action_row == patch_position.0 && action_column == patch_position.1 {
                    return Ok(*action);
                }
            }

            prompt = format!(
                "Position ({}, {}) is not valid. Please enter a valid position ({}). {}",
                row,
                column,
                valid_actions
                    .iter()
                    .map(|action| {
                        if action.is_special_patch_placement() {
                            let action_row = action.get_row();
                            let action_column = action.get_column();
                            format!("({}, {})", action_row + 1, action_column + 1)
                        } else {
                            String::new()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                initial_prompt
            );
        }
    }

    /// Gets the input for the normal action.
    ///
    /// # Arguments
    ///
    /// * `state` - The current state.
    /// * `valid_actions` - The valid actions.
    ///
    /// # Returns
    ///
    /// The action.
    fn handle_normal_action(&mut self, state: &Patchwork, valid_actions: Vec<ActionId>) -> PlayerResult<ActionId> {
        let mut valid_actions = valid_actions;
        let mut actions: HashSet<&str, _> = HashSet::new();
        actions.insert("walk");

        for action in &valid_actions {
            if action.is_first_patch_taken() {
                actions.insert("take 1");
            } else if action.is_second_patch_taken() {
                actions.insert("take 2");
            } else if action.is_third_patch_taken() {
                actions.insert("take 3");
            }
        }

        let mut available_actions = actions.iter().map(|a| format!("'{a}'")).collect::<Vec<_>>();
        available_actions.sort_unstable();

        let initial_prompt = format!(
            "Player '{}' can choose one of the following actions: {}. Please enter the action:",
            self.name,
            available_actions.join(", ")
        );
        #[allow(clippy::redundant_clone)] // This clone is needed but clippy does not get this
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt)?;

            if human_input == "skip" {
                let index = rand::thread_rng().gen_range(0..valid_actions.len());
                return Ok(valid_actions.remove(index));
            }

            if human_input == "walk" {
                return Ok(valid_actions.remove(0));
            }

            if human_input == "take 1" && actions.contains("take 1") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_first_patch_taken())
                        .copied()
                        .collect::<Vec<ActionId>>()
                        .as_slice(),
                    0,
                );
            } else if human_input == "take 2" && actions.contains("take 2") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_second_patch_taken())
                        .copied()
                        .collect::<Vec<ActionId>>()
                        .as_slice(),
                    1,
                );
            } else if human_input == "take 3" && actions.contains("take 3") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_third_patch_taken())
                        .copied()
                        .collect::<Vec<ActionId>>()
                        .as_slice(),
                    2,
                );
            }

            prompt = format!("Please enter a valid action. {initial_prompt}");
        }
    }

    /// Gets the input for the patch placement.
    ///
    /// # Arguments
    ///
    /// * `state` - The current state.
    /// * `valid_actions` - The valid actions.
    fn handle_place_patch(
        &mut self,
        state: &Patchwork,
        valid_actions: &[ActionId],
        patch_index: u8,
    ) -> PlayerResult<ActionId> {
        let initial_prompt = format!("You chose to place the following patch: \n{}\nPlease enter the  rotation (0, 90, 180, 270) and orientation (if flipped: y/n) of the patch:", state.patches[patch_index as usize]);

        #[allow(clippy::redundant_clone)] // This clone is needed but clippy does not get this
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt)?;

            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'rotation, orientation'. {initial_prompt}");
                continue;
            }

            let optional_rotation = human_inputs[0].parse::<usize>();
            let optional_orientation = human_inputs[1].parse::<char>();

            if optional_rotation.is_err() {
                prompt = format!("Please enter a valid number for rotation (0, 90, 180, 270). {initial_prompt}");
                continue;
            }

            let rotation = optional_rotation.unwrap();
            let orientation = optional_orientation.unwrap_or('x');

            if orientation != 'y' && orientation != 'n' {
                prompt = format!("Please enter 'y' or 'n' for orientation. {initial_prompt}");
                continue;
            }

            let rotation = match rotation {
                0 => PatchTransformation::ROTATION_0,
                90 => PatchTransformation::ROTATION_90,
                180 => PatchTransformation::ROTATION_180,
                270 => PatchTransformation::ROTATION_270,
                _ => {
                    prompt = format!("Please enter a valid number for rotation (0, 90, 180, 270). {initial_prompt}");
                    continue;
                }
            };

            let orientation: u8 = u8::from(orientation != 'n');

            let new_valid_actions = valid_actions
                .iter()
                .filter(|action| {
                    if action.is_patch_placement() {
                        let index = action.get_patch_index();
                        let patch_id = action.get_patch_id();
                        let patch_transformation_index = action.get_patch_transformation_index();
                        let transformation = PatchManager::get_transformation(patch_id, patch_transformation_index);

                        index == patch_index
                            && rotation == transformation.rotation_flag()
                            && orientation == transformation.orientation_flag()
                    } else {
                        false
                    }
                })
                .copied()
                .collect::<Vec<ActionId>>();

            if !new_valid_actions.is_empty() {
                return self.handle_place_patch_position(&new_valid_actions);
            }

            prompt = format!(
                "Rotation '{rotation}' and Orientation '{orientation}' is not valid. Please enter a valid rotation and orientation. {initial_prompt}"
            );
        }
    }

    /// Gets the input for the patch placement position.
    ///
    /// # Arguments
    ///
    /// * `valid_actions` - The valid actions.
    ///
    /// # Returns
    ///
    /// The action.
    fn handle_place_patch_position(&mut self, valid_actions: &[ActionId]) -> PlayerResult<ActionId> {
        let initial_prompt = "Please enter the row and column of the patch (row, column):";
        let mut prompt = initial_prompt.to_string();

        loop {
            let human_input = self.get_human_input(&prompt)?;
            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'row, column'. {initial_prompt}");
                continue;
            }

            let optional_row = human_inputs[0].parse::<u8>();
            let optional_column = human_inputs[1].parse::<u8>();

            if optional_row.is_err() || optional_column.is_err() {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let row = optional_row.unwrap();
            let column = optional_column.unwrap();

            if row > QuiltBoard::ROWS || column > QuiltBoard::COLUMNS || row == 0 || column == 0 {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let patch_position = (row - 1, column - 1);

            for action in valid_actions {
                if action.is_patch_placement() {
                    let action_row = action.get_row();
                    let action_column = action.get_column();

                    if action_row == patch_position.0 && action_column == patch_position.1 {
                        return Ok(*action);
                    }
                }
            }

            prompt = format!(
                "Position ({}, {}) is not valid. Please enter a valid position ({}). {}",
                row,
                column,
                valid_actions
                    .iter()
                    .map(|action| {
                        if action.is_patch_placement() {
                            let action_row = action.get_row();
                            let action_column = action.get_column();
                            format!("({}, {})", action_row + 1, action_column + 1)
                        } else {
                            String::new()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                initial_prompt
            );
        }
    }

    fn get_human_input(&mut self, prompt: &str) -> PlayerResult<String> {
        let mut human_input = String::new();
        print!("{prompt} ");
        io::stdout().lock().flush().unwrap();
        io::stdin().read_line(&mut human_input)?;

        human_input = human_input.trim().to_lowercase();
        self.handle_exit_input(&human_input);

        Ok(human_input)
    }

    /// Handles the exit input.
    ///
    /// # Arguments
    ///
    /// * `human_input` - The human input.
    #[allow(clippy::unused_self)]
    fn handle_exit_input(&mut self, human_input: &str) {
        if human_input == "exit" {
            println!("Exiting...");
            std::process::exit(0);
        }
    }
}
