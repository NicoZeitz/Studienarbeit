import numpy as np

from collections import namedtuple
from typing import Literal, Self


from .entities_enum import EntitiesEnum

PlayerPosition = namedtuple('PlayerPosition', 'current_player other_player')

class TimeBoard:
    def __init__(self):
        self.board = np.zeros(54, dtype=np.uint8)

        self.board[0] = EntitiesEnum.PLAYER_1 | EntitiesEnum.PLAYER_2

        self.board[5] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[11] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[17] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[23] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[29] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[35] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[41] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[47] = EntitiesEnum.BUTTON_INCOME_TRIGGER
        self.board[53] = EntitiesEnum.BUTTON_INCOME_TRIGGER

        self.board[26] = EntitiesEnum.SPECIAL_PATCH
        self.board[32] = EntitiesEnum.SPECIAL_PATCH
        self.board[38] = EntitiesEnum.SPECIAL_PATCH
        self.board[44] = EntitiesEnum.SPECIAL_PATCH
        self.board[50] = EntitiesEnum.SPECIAL_PATCH

    def get_player_positions(self, current_player: Literal[EntitiesEnum.PLAYER_1, EntitiesEnum.PLAYER_2] = EntitiesEnum.PLAYER_1) -> PlayerPosition:

        other_player = EntitiesEnum.PLAYER_2 if current_player == EntitiesEnum.PLAYER_1 else EntitiesEnum.PLAYER_1

        current_player_position = np.where((self.board & current_player) > 0)[0][0]
        other_player_position = np.where((self.board & other_player) > 0)[0][0]

        # FIXME: remove in build
        assert current_player_position <= other_player_position, f"Current player (pos: {current_player_position}) has to be before or on the same position as the other player (pos: {other_player_position})"

        return PlayerPosition(current_player_position, other_player_position)

    def set_player_position(self, current_player: Literal[EntitiesEnum.PLAYER_1, EntitiesEnum.PLAYER_2], position: int):
        self.board = np.where((self.board & current_player) > 0, 0, self.board)
        self.board[position] = self.board[position] | current_player

    def copy(self) -> Self:
        new_patchwork_game_board = TimeBoard()
        new_patchwork_game_board.board = self.board.copy()
        return new_patchwork_game_board

    def __repr__(self):
        board = self.board.tolist()
        for index, field in enumerate(board):
            display_str = []
            if field & EntitiesEnum.PLAYER_1 > 0:
                display_str.append("Player 1")
            if field & EntitiesEnum.PLAYER_2 > 0:
                display_str.append("Player 2")
            if field & EntitiesEnum.BUTTON_INCOME_TRIGGER > 0:
                display_str.append("Button")
            if field & EntitiesEnum.SPECIAL_PATCH > 0:
                display_str.append("Special Patch")
            board[index] = ', '.join(display_str)
        return str(board)