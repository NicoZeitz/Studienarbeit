from collections import namedtuple
from copy import deepcopy
from enum import IntFlag
from typing import Literal, Self

import numpy as np
import numpy.typing as npt

PlayerPosition = namedtuple('PlayerPosition', 'current_player other_player')

class EntitiesEnum(IntFlag):
    PLAYER_1              = 0b0001
    PLAYER_2              = 0b0010
    BUTTON_INCOME_TRIGGER = 0b0100
    SPECIAL_PATCH         = 0b1000

# TODO: BUG: Player Positions at start are not correct/ wrongly displayed
class TimeBoard:
    """The time board of the game."""

    # ================================ instance attributes ================================

    tiles: np.ndarray((54,), np.uint8)
    """The tiles of the time board."""

    # ================================ static attributes ================================

    MAX_POSITION = 53
    """The maximum position on the time board."""

    # ================================ static methods ================================

    @staticmethod
    def initial_board() -> Self:
        """Returns the initial time board."""
        tiles = np.zeros(54, dtype=np.uint8)
        tiles[0] = EntitiesEnum.PLAYER_1 | EntitiesEnum.PLAYER_2

        tiles[5] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[11] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[17] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[23] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[29] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[35] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[41] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[47] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        tiles[53] = EntitiesEnum.BUTTON_INCOME_TRIGGER

        tiles[26] = EntitiesEnum.SPECIAL_PATCH
        tiles[32] = EntitiesEnum.SPECIAL_PATCH
        tiles[38] = EntitiesEnum.SPECIAL_PATCH
        tiles[44] = EntitiesEnum.SPECIAL_PATCH
        tiles[50] = EntitiesEnum.SPECIAL_PATCH
        return TimeBoard(tiles)

    # ================================ instance methods ================================

    def __init__(self, tiles: np.ndarray((54,), np.uint8)):
        self.tiles = tiles

    # special patches

    def get_special_patches_in_range(self, range: range) -> npt.NDArray[np.intp]:
        return np.argwhere((self.tiles[self.clamp_range(range)] & EntitiesEnum.SPECIAL_PATCH) > 0) + range.start

    def clear_special_patch(self, index: int):
        clamped_index = self.clamp_index(index)
        self.tiles[clamped_index] = self.tiles[clamped_index] ^ EntitiesEnum.SPECIAL_PATCH

    # button income triggers

    def get_amount_button_income_triggers_in_range(self, range: range) -> int:
        return np.count_nonzero((self.tiles[self.clamp_range(range)] & EntitiesEnum.BUTTON_INCOME_TRIGGER) > 0)

    # player positions

    def set_player_position(self, player: Literal[EntitiesEnum.PLAYER_1, EntitiesEnum.PLAYER_2], old_position: int, new_position: int) -> None:
        # TODO:PERF: defensive copy as the copy() method uses a view of the array for performance
        self.tiles = self.tiles.copy()

        # reset old position
        self.tiles[old_position] ^= player

        # set new position
        clamped_position = self.clamp_index(new_position)
        self.tiles[clamped_position] |= player

    # other function

    def clamp_range(self, input_range: range) -> range:
        return range(
            max(input_range.start, 0),
            min(input_range.stop, TimeBoard.MAX_POSITION + 1)
        )

    def clamp_index(self, index: int) -> int:
        return min(max(index, 0), TimeBoard.MAX_POSITION)

    def __eq__(self, o: object) -> bool:
        return self.tiles == o.tiles

    def __hash__(self) -> int:
        return hash(self.tiles)

    def __repr__(self) -> str:
        return f'TimeBoard(tiles={self.tiles})'

    def __str__(self) -> str:
        def get_str_for_tile(tile: int) -> str:
            result_str = ''

            if tile & EntitiesEnum.PLAYER_1 > 0:
                result_str += '1'
            if tile & EntitiesEnum.PLAYER_2 > 0:
                result_str += '2'
            if tile & EntitiesEnum.BUTTON_INCOME_TRIGGER > 0:
                result_str += 'B'
            elif tile & EntitiesEnum.SPECIAL_PATCH > 0:
                result_str += 'P'

            if len(result_str) == 0:
                result_str = ' '

            return result_str

        first_line =  []
        second_line = []
        third_line =  []

        for field in self.tiles:
            tile_str = get_str_for_tile(field)

            first_line.append('─' * len(tile_str))
            second_line.append(tile_str)
            third_line.append('─' * len(tile_str))

        result_str =  '┌' + '┬'.join(first_line)  + '┐\n'
        result_str += '│' + '│'.join(second_line) + '│\n'
        result_str += '└' + '┴'.join(third_line)  + '┘'
        return result_str

    def __copy__(self) -> Self:
        return TimeBoard(self.tiles)

    def __deepcopy__(self, memo: dict) -> Self:
        return TimeBoard(deepcopy(self.tiles, memo))

    def copy(self) -> Self:
        return TimeBoard(self.tiles)