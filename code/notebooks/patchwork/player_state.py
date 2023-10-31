from copy import deepcopy
from typing import Self, Optional

from .quilt_board import QuiltBoard

class PlayerState:
    """A player in the game of Patchwork."""

    # ================================ instance attributes ================================
    name: Optional[str]
    """The name of the player."""

    position: int
    """The position of the player on the time board."""

    button_balance: int
    """The amount of buttons the player has."""

    quilt_board: QuiltBoard
    """The quilt board of the player."""

    # ================================ instance methods ================================

    def __init__(self, name: Optional[str], position: int, button_balance: int, quilt_board: QuiltBoard) -> Self:
        self.name = name
        self.position = position
        self.button_balance = button_balance
        self.quilt_board = quilt_board

    def __eq__(self, other: Self) -> bool:
        return self.name == other.name and \
            self.position == other.position and \
            self.button_balance == other.button_balance and \
            self.quilt_board == other.quilt_board

    def __hash__(self) -> int:
        return hash((
            self.name,
            self.position,
            self.button_balance,
            self.quilt_board
        ))

    def __repr__(self) -> str:
        return f'Player(name={self.name}, position={self.position}, button_balance={self.button_balance}, quilt_board={self.quilt_board})'

    def __str__(self) -> str:
        player_str = f'Player \'{self.name if self.name is not None else "Unknown"}\' (button balance: {self.button_balance}):\n'
        player_str += f'{self.quilt_board}'
        return player_str

    def __copy__(self) -> Self:
        return PlayerState(
            name=self.name,
            position=self.position,
            button_balance=self.button_balance,
            quilt_board=self.quilt_board
        )

    def __deepcopy__(self, memo: dict) -> Self:
        return PlayerState(
            name=self.name,
            position=self.position,
            button_balance=self.button_balance,
            quilt_board=deepcopy(self.quilt_board, memo)
        )

    def copy(self) -> Self:
         return PlayerState(
            name=self.name,
            position=self.position,
            button_balance=self.button_balance,
            quilt_board=self.quilt_board.copy()
        )