from dataclasses import dataclass
from typing import List, Literal, Self

from .entities_enum import EntitiesEnum
from .quilt_board import QuiltBoard
from .patch import Patch
from .time_board import TimeBoard

@dataclass
class State:
    pieces: List[Patch]
    time_board: TimeBoard
    player_1_quilt_board: QuiltBoard
    player_2_quilt_board: QuiltBoard
    current_player: Literal[EntitiesEnum.PLAYER_1, EntitiesEnum.PLAYER_2]
    player_1_button_balance: int
    player_2_button_balance: int

    # getters regarding all players

    @property
    def player_positions(self):
        return self.time_board.get_player_positions(self.current_player)

    # getters for the current player

    @property
    def current_player_quilt_board(self) -> QuiltBoard:
        return self.player_1_quilt_board if self.current_player == EntitiesEnum.PLAYER_1 else self.player_2_quilt_board

    @current_player_quilt_board.setter
    def current_player_quilt_board(self, value: QuiltBoard) -> None:
        if self.current_player == EntitiesEnum.PLAYER_1:
            self.player_1_quilt_board = value
        else:
            self.player_2_quilt_board = value

    @property
    def current_player_button_balance(self) -> int:
        return self.player_1_button_balance if self.current_player == EntitiesEnum.PLAYER_1 else self.player_2_button_balance

    @property
    def current_player_button_income(self) -> int:
        return self.current_player_quilt_board.button_income

    # setters for the current player

    def set_current_player_position(self, position: int) -> None:
        self.time_board.set_player_position(self.current_player, position)

    def set_current_player_button_balance(self, new_button_balance: int) -> None:
        if self.current_player == EntitiesEnum.PLAYER_1:
            self.player_1_button_balance = new_button_balance
        else:
            self.player_2_button_balance = new_button_balance

    def add_current_player_button_balance(self, amount: int) -> None:
        self.set_current_player_button_balance(self.current_player_button_balance + amount)

    def subtract_current_player_button_balance(self, amount: int) -> None:
        self.set_current_player_button_balance(self.current_player_button_balance - amount)

    # setters regarding all players

    def switch_current_player(self) -> None:
        self.current_player = EntitiesEnum.PLAYER_1 if self.current_player == EntitiesEnum.PLAYER_2 else EntitiesEnum.PLAYER_2

    # utility methods

    def copy(self) -> Self:
        new_state = State(
            pieces=self.pieces.copy(),
            time_board=self.time_board.copy(),
            player_1_quilt_board=self.player_1_quilt_board.copy(),
            player_2_quilt_board=self.player_2_quilt_board.copy(),
            current_player=self.current_player,
            player_1_button_balance=self.player_1_button_balance,
            player_2_button_balance=self.player_2_button_balance
        )
        return new_state