from typing import List
import random
import re

from ...action import Action, Position as PatchPosition
from ...game import Game
from ...player import Player
from ...state import State
from ...quilt_board import QuiltBoard
from ...patch import Rotation, Orientation

class HumanPlayer(Player):
    """
    A player that is a human.
    """

    def get_action(
            self,
            game: Game,
            state: State
    ) -> Action:
        valid_actions = game.get_valid_actions(state)

        if valid_actions[0].is_special_patch_placement:
            return self.handle_special_patch_action(state, valid_actions)

        return self.handle_normal_action(state, valid_actions)

    def handle_special_patch_action(self, state: State, valid_actions: List[Action]) -> Action:
        """
        Gets the input for the special patch placement.

        :param state: The current state.
        :param valid_actions: The valid actions.
        :return: The action.
        """

        initial_prompt = f"Player '{state.current_player.name}' has to place the special patch. Please enter the row and column of the patch (row, column):"

        prompt = initial_prompt

        while True:
            human_input = input(prompt).lower()
            self.handle_exit_input(human_input)

            if human_input == "skip":
                return random.choice(valid_actions)

            human_inputs = re.split("[, ]+", human_input)

            if len(human_inputs) != 2:
                prompt = "Please enter 'row, column'. " + initial_prompt
                continue

            if not human_inputs[0].isdigit() or not human_inputs[1].isdigit():
                prompt = f"Please enter valid numbers for row (1-{QuiltBoard.ROWS}) and column (1-{QuiltBoard.COLUMNS}). " + initial_prompt
                continue

            row = int(human_inputs[0])
            column = int(human_inputs[1])

            if row > QuiltBoard.ROWS or column > QuiltBoard.COLUMNS:
                prompt = f"Please enter valid numbers for row (1-{QuiltBoard.ROWS}) and column (1-{QuiltBoard.COLUMNS}). " + initial_prompt
                continue

            patch_position = PatchPosition(row - 1, column - 1)

            for action in valid_actions:
                if action.patch_position == patch_position:
                    return action


            prompt = f"Position ({row}, {column}) is not valid. Please enter a valid position ({list(map(lambda a: a.patch_position, valid_actions))}). " + initial_prompt

    def handle_normal_action(self, state: State, valid_actions: List[Action]) -> Action:
        """
        Gets the input for the normal action.

        :param state: The current state.
        :param valid_actions: The valid actions.
        :return: The action.
        """

        actions = { 'walk' }

        for action in valid_actions:
            if action.is_first_patch_taken:
                actions.add('take 1')
            elif action.is_second_patch_taken:
                actions.add('take 2')
            elif action.is_third_patch_taken:
                actions.add('take 3')

        initial_prompt = f"Player '{state.current_player.name}' can chose one of the following actions: {', '.join(sorted(actions))}. Please enter the action:"

        prompt = initial_prompt

        while True:
            human_input = input(prompt).lower()
            self.handle_exit_input(human_input)

            if human_input == "skip":
                return random.choice(valid_actions)

            if human_input == "walk":
                return valid_actions[0]

            if human_input == "take 1":
                return self.handle_place_patch(state, list(filter(lambda action: action.patch_index == 0, valid_actions)), 0)
            elif human_input == "take 2":
                return self.handle_place_patch(state, list(filter(lambda action: action.patch_index == 1, valid_actions)), 1)
            elif human_input == "take 3":
                return self.handle_place_patch(state, list(filter(lambda action: action.patch_index == 2, valid_actions)), 2)

            prompt = f"Please enter a valid action. " + initial_prompt

    def handle_place_patch(self, state: State, valid_actions: List[Action], patch_index: int) -> Action:
        """
        Gets the input for the patch placement.

        :param state: The current state.
        :param valid_actions: The valid actions.
        :param patch_index: The patch index.
        """

        initial_prompt = f'You chose to place the following patch: \n{state.patches[patch_index]}\nPlease enter the  rotation (0, 90, 180, 270) and orientation (if flipped: y/n) of the patch:'

        prompt = initial_prompt

        while True:
            human_input = input(prompt).lower()
            self.handle_exit_input(human_input)

            human_inputs = re.split("[, ]+", human_input)

            if len(human_inputs) != 2:
                prompt = "Please enter 'rotation, orientation'. " + initial_prompt
                continue

            if not human_inputs[0].isdigit():
                prompt = f"Please enter a valid number for rotation (0, 90, 180, 270). " + initial_prompt
                continue

            if human_inputs[1] != 'y' and human_inputs[1] != 'n':
                prompt = f"Please enter 'y' or 'n' for orientation. " + initial_prompt
                continue

            rotation = int(human_inputs[0])
            if rotation == 0:
                rotation = Rotation.ZERO
            elif rotation == 90:
                rotation = Rotation.NINETY
            elif rotation == 180:
                rotation = Rotation.ONE_EIGHTY
            elif rotation == 270:
                rotation = Rotation.TWO_SEVENTY
            else:
                prompt = f"Please enter a valid number for rotation (0, 90, 180, 270). " + initial_prompt
                continue

            orientation = Orientation.FLIPPED if human_inputs[1] == 'y' else Orientation.NORMAL

            now_valid_actions = list(filter(lambda action: action.patch_index == patch_index and action.patch.rotation == rotation and action.patch.orientation == orientation, valid_actions))

            if len(now_valid_actions) > 0:
                return self.handle_place_patch_position(now_valid_actions)

            prompt = f"Rotation '{rotation}' and Orientation '{orientation}' is not valid. Please enter a valid rotation and orientation. " + initial_prompt

    def handle_place_patch_position(self, valid_actions: List[Action]) -> Action:
        """
        Gets the input for the patch placement position.

        :param valid_actions: The valid actions.
        :return: The action.
        """

        initial_prompt = f"Please enter the row and column of the patch (row, column):"

        prompt = initial_prompt

        while True:
            human_input = input(prompt).lower()
            self.handle_exit_input(human_input)

            human_inputs = re.split("[, ]+", human_input)

            if len(human_inputs) != 2:
                prompt = "Please enter 'row, column'. " + initial_prompt
                continue

            if not human_inputs[0].isdigit() or not human_inputs[1].isdigit():
                prompt = f"Please enter valid numbers for row (1-{QuiltBoard.ROWS}) and column (1-{QuiltBoard.COLUMNS}). " + initial_prompt
                continue

            row = int(human_inputs[0])
            column = int(human_inputs[1])

            if row > QuiltBoard.ROWS or column > QuiltBoard.COLUMNS:
                prompt = f"Please enter valid numbers for row (1-{QuiltBoard.ROWS}) and column (1-{QuiltBoard.COLUMNS}). " + initial_prompt
                continue

            patch_position = PatchPosition(row - 1, column - 1)

            for action in valid_actions:
                if action.patch_position == patch_position:
                    return action

            prompt = f"Position ({row}, {column}) is not valid. Please enter a valid position ({list(map(lambda a: a.patch_position, valid_actions))}). " + initial_prompt

    def handle_exit_input(self, human_input: str) -> None:
        """
        Handles the exit input.

        :param human_input: The human input.
        :raises KeyboardInterrupt: If the human input is 'exit'.
        """

        if human_input == "exit":
            raise KeyboardInterrupt()