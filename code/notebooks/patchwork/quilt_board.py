from typing import List, Literal, Self, Optional

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view

from .patch import Patch
from .action import Action, Position as PatchPosition

class QuiltBoard:
    """The quilt board of the player."""

    # ================================ instance attributes ================================

    tiles: np.ndarray[(9,9), np.bool_]
    """The tiles of the board."""

    button_income: int = 0
    """The amount of buttons this board generates."""

    is_full: bool = False
    """Whether the board is full."""

    # ================================ instance properties ================================

    @property
    def score(self) -> int:
        """The score the player has with this quilt board."""
        return -2 * np.count_nonzero(self.tiles == 0)

    # ================================ instance methods ================================

    # ================================ instance methods ================================

    def __init__(self):
        self.tiles = np.zeros((9,9), dtype=bool)

    def add_patch(self, patch: Patch, position: PatchPosition) -> None:
        """Adds a patch to the quilt board at the given position."""
        self.button_income += patch.button_income
        self.tiles[
            position[0] : position[0] + patch.tiles.shape[0],
            position[1] : position[1] + patch.tiles.shape[1]
        ] |= patch.tiles

        self.is_full = np.all(self.tiles)

    def get_valid_actions_for_patch(
            self,
            patch: Patch,
            patch_index: Optional[Literal[0,1,2]] = None
    ) -> List[Action]:
        """
        Returns all valid actions for the given patch and the current quilt board state.

        :param patch: The patch to get the valid actions for.
        :param patch_index: The index of the patch from the list of available patches.
        :return: A list of valid actions for the given patch.
        """

        if self.is_full:
            return []

        valid_actions_for_patch = []

        sliding_board_window = sliding_window_view(self.tiles, patch.shape)
        sliding_board_window_rotated_patch = None
        if patch.shape[0] != patch.shape[1]:
            sliding_board_window_rotated_patch = sliding_window_view(self.tiles, (patch.shape[1], patch.shape[0]))

        for transformed_patch in patch.get_unique_transformations():
            board_window = sliding_board_window if patch.shape[0] == transformed_patch.shape[0] else sliding_board_window_rotated_patch

            patch_window = np.repeat(
                np.repeat(transformed_patch.tiles[np.newaxis, :, :], board_window.shape[1], axis=0)[np.newaxis, :, :],
                board_window.shape[0], axis=0
            )

            combined_windows = np.bitwise_and.reduce(np.bitwise_not(np.bitwise_and(board_window, patch_window)), axis=(2,3))

            for (row, column) in np.argwhere(combined_windows):
                valid_actions_for_patch.append(Action(
                    transformed_patch,
                    PatchPosition(row, column),
                    patch_index
                ))

        return valid_actions_for_patch

        # old implementation
        # for transformed_patch in patch.get_unique_transformations():
        #     (transformed_row, transformed_column) = transformed_patch.shape

        #     rows = np.size(self.tiles, 0) - transformed_row + 1
        #     columns = np.size(self.tiles, 1) - transformed_column + 1

        #     for (row, column) in np.ndindex(rows, columns):
        #         board_tiles_view = self.tiles[
        #             row    : row    + transformed_row,
        #             column : column + transformed_column
        #         ]
        #         combination = (transformed_patch.tiles | board_tiles_view)

        #         ones_in_patch = np.count_nonzero(transformed_patch.tiles)
        #         ones_in_board_tiles_view = np.count_nonzero(board_tiles_view)
        #         ones_in_combination = np.count_nonzero(combination)

        #         if ones_in_combination != ones_in_board_tiles_view + ones_in_patch:
        #             continue

        #         valid_actions_for_patch.append(Action(
        #             transformed_patch,
        #             PatchPosition(row, column),
        #             patch_index
        #         ))

        # return valid_actions_for_patch

    def __eq__(self, other: Self) -> bool:
        return self.tiles == other.tiles and \
            self.button_income == other.button_income

    def __hash__(self) -> int:
        return hash((
            self.tiles,
            self.button_income
        ))

    def __repr__(self) -> str:
        return f'QuiltBoard(board={self.tiles}, button_income={self.button_income})'

    def __str__(self) -> str:
        quilt_board_str = ''
        for row in range(0, np.size(self.tiles, 0)):
            for column in range(0, np.size(self.tiles, 1)):
                quilt_board_str += '█' if self.tiles[row, column] else '░'
            quilt_board_str += '\n'
        quilt_board_str += f'Button income: {self.button_income}'
        return quilt_board_str

    def __copy__(self) -> Self:
        copy = QuiltBoard()
        copy.tiles = self.tiles.copy()
        copy.button_income = self.button_income
        return copy

    def copy(self) -> Self:
        return self.__copy__()