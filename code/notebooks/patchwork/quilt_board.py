from copy import deepcopy
from typing import Any, ClassVar, List, Literal, Mapping, Optional, Self, Union

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view

from .action import Action, Position as PatchPosition
from .patch import Patch

class QuiltBoard:
    """The quilt board of the player."""

    __slots__ = ('tiles', 'button_income', 'is_full', 'tiles_filled')

    # ================================ static attributes ================================

    ROWS: ClassVar[Literal[9]] = 9
    """The amount of rows on the quilt board."""

    COLUMNS: ClassVar[Literal[9]] = 9
    """The amount of columns on the quilt board."""

    TILES: ClassVar[Literal[81]] = ROWS * COLUMNS
    """The amount of tiles on the quilt board."""

    # ================================ attributes ================================

    tiles: np.ndarray[(9,9), np.bool_]
    """The tiles of the board."""

    button_income: int
    """The amount of buttons this board generates."""

    is_full: bool
    """Whether the board is full."""

    tiles_filled: int
    """The amount of tiles that are filled."""

    # ================================ properties ================================

    @property
    def score(self) -> int:
        """The score the player has with this quilt board."""
        return -2 * np.count_nonzero(self.tiles == 0)

    @property
    def percentage_filled(self) -> float:
        """The percentage of tiles that are filled."""
        return self.tiles_filled / QuiltBoard.TILES

    # ================================ constructor ================================

    def __init__(self, button_income: int, tiles: np.ndarray[(9,9), np.bool_], *, tiles_filled: int = 0):
        self.button_income = button_income
        self.tiles = tiles
        self.tiles_filled = tiles_filled
        self.is_full = self.tiles_filled == QuiltBoard.TILES

    # ================================ static methods ================================

    @staticmethod
    def empty_board() -> Self:
        """Returns an empty quilt board."""
        return QuiltBoard(0, np.zeros((QuiltBoard.ROWS, QuiltBoard.COLUMNS), dtype=bool))

    # ================================ methods ================================

    def add_patch(self, patch: Patch, position: PatchPosition) -> None:
        """Adds a patch to the quilt board at the given position."""
        self.button_income += patch.button_income

        # TODO:PERF: defensive copy as the copy() method uses a view of the array for performance
        self.tiles = self.tiles.copy()
        self.tiles[
            position[0] : position[0] + patch.tiles.shape[0],
            position[1] : position[1] + patch.tiles.shape[1]
        ] |= patch.tiles
        self.tiles_filled += np.count_nonzero(patch.tiles)
        self.is_full = self.tiles_filled == QuiltBoard.TILES

    def is_valid_patch_placement(self, patch: Patch, position: PatchPosition) -> bool:
        """
        Tests whether the given patch can be placed at the given position on the quilt board.
        """
        return np.any(self.tiles[
            position[0] : position[0] + patch.tiles.shape[0],
            position[1] : position[1] + patch.tiles.shape[1]
        ] & patch.tiles)

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

        # TODO: remove old implementation
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

    def __eq__(self, other: Any) -> Union[NotImplemented, bool]:
        if not isinstance(other, QuiltBoard):
            return NotImplemented

        return self.tiles == other.tiles and \
            self.button_income == other.button_income

    def __hash__(self) -> int:
        return hash((
            self.tiles,
            self.button_income
        ))

    def __repr__(self) -> str:
        return f'{type(self)}(board={self.tiles}, button_income={self.button_income})'

    def __str__(self) -> str:
        quilt_board_str = ''
        for row in range(0, QuiltBoard.ROWS):
            for column in range(0, QuiltBoard.COLUMNS):
                quilt_board_str += '█' if self.tiles[row, column] else '░'
            quilt_board_str += '\n'
        quilt_board_str += f'Button income: {self.button_income}'
        return quilt_board_str

    def __copy__(self) -> Self:
        return QuiltBoard(self.button_income, self.tiles, tiles_filled=self.tiles_filled)

    def __deepcopy__(self, memo: Mapping) -> Self:
        return QuiltBoard(self.button_income, deepcopy(self.tiles, memo), tiles_filled=self.tiles_filled)

    def copy(self) -> Self:
        return QuiltBoard(self.button_income, self.tiles, tiles_filled=self.tiles_filled)