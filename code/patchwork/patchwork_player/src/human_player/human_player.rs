use std::{
    collections::HashSet,
    io::{self, Write},
};

use patchwork_core::{
    Action, ActionPayload, Game, PatchTransformation, Patchwork, Player, QuiltBoard,
};
use rand::Rng;
use regex::Regex;

/// A player that is human
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HumanPlayer {
    pub name: String,
}

impl Player for HumanPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_action(&mut self, game: &Patchwork) -> Action {
        let valid_actions = game.get_valid_actions();

        match valid_actions[0].payload {
            ActionPayload::SpecialPatchPlacement { payload: _ } => {
                self.handle_special_patch_action(valid_actions)
            }
            _ => self.handle_normal_action(game, valid_actions),
        }
    }
}

impl HumanPlayer {
    pub fn new(name: String) -> Self {
        HumanPlayer { name }
    }

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
    fn handle_special_patch_action(&mut self, valid_actions: Vec<Action>) -> Action {
        let mut valid_actions = valid_actions;
        let initial_prompt = format!("Player '{}' has to place the special patch. Please enter the row and column of the patch (row, column):", self.name);
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt);

            if human_input == "skip" {
                let index = rand::thread_rng().gen_range(0..valid_actions.len());
                return valid_actions.remove(index);
            }

            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'row, column'. {}", initial_prompt);
                continue;
            }

            let optional_row = human_inputs[0].parse::<usize>();
            let optional_column = human_inputs[1].parse::<usize>();

            if optional_row.is_err() || optional_column.is_err() {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let row: usize = optional_row.unwrap();
            let column: usize = optional_column.unwrap();

            if row > QuiltBoard::ROWS as usize || column > QuiltBoard::COLUMNS as usize {
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
                if let ActionPayload::SpecialPatchPlacement { payload } = &action.payload {
                    if payload.row == patch_position.0 && payload.column == patch_position.1 {
                        return action.clone();
                    }
                }
            }

            prompt = format!(
                "Position ({}, {}) is not valid. Please enter a valid position ({}). {}",
                row,
                column,
                valid_actions
                    .iter()
                    .map(|a| match &a.payload {
                        ActionPayload::SpecialPatchPlacement { payload } =>
                            format!("({}, {})", payload.row, payload.column),
                        _ => "".to_string(),
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
    fn handle_normal_action(&mut self, state: &Patchwork, valid_actions: Vec<Action>) -> Action {
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

        let mut available_actions = actions
            .iter()
            .map(|a| format!("'{}'", a))
            .collect::<Vec<_>>();
        available_actions.sort_unstable();

        let initial_prompt = format!(
            "Player '{}' can choose one of the following actions: {}. Please enter the action:",
            self.name,
            available_actions.join(", ")
        );
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt);

            if human_input == "skip" {
                let index = rand::thread_rng().gen_range(0..valid_actions.len());
                return valid_actions.remove(index);
            }

            if human_input == "walk" {
                return valid_actions.remove(0);
            }

            if human_input == "take 1" && actions.contains("take 1") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_first_patch_taken())
                        .cloned()
                        .collect::<Vec<Action>>(),
                    0,
                );
            } else if human_input == "take 2" && actions.contains("take 2") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_second_patch_taken())
                        .cloned()
                        .collect::<Vec<Action>>(),
                    1,
                );
            } else if human_input == "take 3" && actions.contains("take 3") {
                return self.handle_place_patch(
                    state,
                    valid_actions
                        .iter()
                        .filter(|action| action.is_third_patch_taken())
                        .cloned()
                        .collect::<Vec<Action>>(),
                    2,
                );
            }

            prompt = format!("Please enter a valid action. {}", initial_prompt);
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
        valid_actions: Vec<Action>,
        patch_index: usize,
    ) -> Action {
        let initial_prompt = format!("You chose to place the following patch: \n{}\nPlease enter the  rotation (0, 90, 180, 270) and orientation (if flipped: y/n) of the patch:", state.patches[patch_index]);
        let mut prompt = initial_prompt.clone();

        loop {
            let human_input = self.get_human_input(&prompt);

            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'rotation, orientation'. {}", initial_prompt);
                continue;
            }

            let optional_rotation = human_inputs[0].parse::<usize>();
            let optional_orientation = human_inputs[1].parse::<char>();

            if optional_rotation.is_err() {
                prompt = format!(
                    "Please enter a valid number for rotation (0, 90, 180, 270). {}",
                    initial_prompt
                );
                continue;
            }

            let rotation = optional_rotation.unwrap();
            let orientation = optional_orientation.unwrap_or('x');

            if orientation != 'y' && orientation != 'n' {
                prompt = format!(
                    "Please enter 'y' or 'n' for orientation. {}",
                    initial_prompt
                );
                continue;
            }

            let rotation = match rotation {
                0 => PatchTransformation::ROTATION_0 as usize,
                90 => PatchTransformation::ROTATION_90 as usize,
                180 => PatchTransformation::ROTATION_180 as usize,
                270 => PatchTransformation::ROTATION_270 as usize,
                _ => {
                    prompt = format!(
                        "Please enter a valid number for rotation (0, 90, 180, 270). {}",
                        initial_prompt
                    );
                    continue;
                }
            };

            let orientation = if orientation == 'n' { 0 } else { 1 };

            let new_valid_actions = valid_actions
                .iter()
                .filter(|action| {
                    if let ActionPayload::PatchPlacement { payload } = &action.payload {
                        payload.patch_index == patch_index
                            && payload.patch_rotation == rotation
                            && payload.patch_orientation == orientation
                    } else {
                        false
                    }
                })
                .cloned()
                .collect::<Vec<Action>>();

            if !new_valid_actions.is_empty() {
                return self.handle_place_patch_position(new_valid_actions);
            }

            prompt = format!("Rotation '{}' and Orientation '{}' is not valid. Please enter a valid rotation and orientation. {}", rotation, orientation, initial_prompt);
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
    fn handle_place_patch_position(&mut self, valid_actions: Vec<Action>) -> Action {
        let initial_prompt = "Please enter the row and column of the patch (row, column):";
        let mut prompt = initial_prompt.to_string();

        loop {
            let human_input = self.get_human_input(&prompt);
            let human_inputs = Regex::new(r"[, ]+")
                .unwrap()
                .split(&human_input)
                .map(|x| x.to_string())
                .collect::<Vec<_>>();

            if human_inputs.len() != 2 {
                prompt = format!("Please enter 'row, column'. {}", initial_prompt);
                continue;
            }

            let optional_row = human_inputs[0].parse::<usize>();
            let optional_column = human_inputs[1].parse::<usize>();

            if optional_row.is_err() || optional_column.is_err() {
                prompt = format!(
                    "Please enter valid numbers for row (1-{}) and column (1-{}). {}",
                    QuiltBoard::ROWS,
                    QuiltBoard::COLUMNS,
                    initial_prompt
                );
                continue;
            }

            let row: usize = optional_row.unwrap();
            let column: usize = optional_column.unwrap();

            if row > QuiltBoard::ROWS as usize
                || column > QuiltBoard::COLUMNS as usize
                || row == 0
                || column == 0
            {
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
                if let ActionPayload::PatchPlacement { payload } = &action.payload {
                    if payload.row == patch_position.0 && payload.column == patch_position.1 {
                        return action.clone();
                    }
                }
            }

            prompt = format!(
                "Position ({}, {}) is not valid. Please enter a valid position ({}). {}",
                row,
                column,
                valid_actions
                    .iter()
                    .map(|a| match &a.payload {
                        ActionPayload::PatchPlacement { payload } =>
                            format!("({}, {})", payload.row + 1, payload.column + 1),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                initial_prompt
            );
        }
    }

    fn get_human_input(&mut self, prompt: &str) -> String {
        let mut human_input = String::new();
        print!("{} ", prompt);
        io::stdout().lock().flush().unwrap();
        io::stdin()
            .read_line(&mut human_input)
            .expect("Failed to read line"); // TODO: use anyhow and better error handling

        human_input = human_input.trim().to_lowercase();
        self.handle_exit_input(&human_input);

        human_input
    }

    /// Handles the exit input.
    ///
    /// # Arguments
    ///
    /// * `human_input` - The human input.
    fn handle_exit_input(&mut self, human_input: &str) {
        if human_input == "exit" {
            println!("Exiting...");
            std::process::exit(0);
        }
    }
}
