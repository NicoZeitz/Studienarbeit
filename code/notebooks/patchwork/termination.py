
from typing import Any, ClassVar, Literal, Optional, Self, Union

TERMINATION_NOT_TERMINATED = Literal['NOT TERMINATED']
TERMINATION_PLAYER_1_WON = Literal['PLAYER 1 WON']
TERMINATION_PLAYER_2_WON = Literal['PLAYER 2 WON']
TERMINATION_DRAW = Literal['DRAW']

class Termination:
    """
    A class representing if the game is terminated and if so who won and with what score.
    """

    __slots__ = ('state', 'player_1_score', 'player_2_score')

    # ================================ static attributes ================================

    NOT_TERMINATED: ClassVar[TERMINATION_NOT_TERMINATED] = TERMINATION_NOT_TERMINATED
    """The game is not terminated."""

    PLAYER_1_WON: ClassVar[TERMINATION_PLAYER_1_WON] = TERMINATION_PLAYER_1_WON
    """Player 1 won."""

    PLAYER_2_WON: ClassVar[TERMINATION_PLAYER_2_WON] = TERMINATION_PLAYER_2_WON
    """Player 2 won."""

    DRAW: ClassVar[TERMINATION_DRAW] = TERMINATION_DRAW
    """The game ended in a draw."""

    # ================================ attributes ================================

    state: Union[TERMINATION_NOT_TERMINATED, TERMINATION_PLAYER_1_WON, TERMINATION_PLAYER_2_WON, TERMINATION_DRAW]
    """The state of the game."""

    player_1_score: Optional[int]
    """The score of player 1."""

    player_2_score: Optional[int]
    """The score of player 2."""

    # ================================ constructor ================================

    def __init__(
            self,
            state: Union[TERMINATION_NOT_TERMINATED, TERMINATION_PLAYER_1_WON, TERMINATION_PLAYER_2_WON, TERMINATION_DRAW],
            *,
            player_1_score: Optional[int] = None,
            player_2_score: Optional[int] = None
    ):
        self.state = state
        self.player_1_score = player_1_score
        self.player_2_score = player_2_score

    # ================================ properties ================================

    @property
    def is_terminated(self) -> bool:
        return self.state != Termination.NOT_TERMINATED

    @property
    def is_draw(self) -> bool:
        return self.state == Termination.DRAW

    @property
    def is_player_1_won(self) -> bool:
        return self.state == Termination.PLAYER_1_WON

    @property
    def is_player_2_won(self) -> bool:
        return self.state == Termination.PLAYER_2_WON

    @property
    def score(self) -> int:
        """
        Returns the score of the game. Positive if player 1 won, negative if player 2 won, 0 if draw.

        :return: The score of the game.
        :raises ValueError: If the game is not terminated.
        """

        if not self.is_terminated:
            raise ValueError("The game is not terminated")

        return self.player_1_score - self.player_2_score

    # ================================ methods ================================

    def __eq__(self, other: Any) -> Union[NotImplemented, bool]:
        if isinstance(other, str):
            return self.state == other

        if not isinstance(other, Termination):
            return NotImplemented

        return self.state == other.state

    def __hash__(self) -> int:
        return hash(self.state)

    def __repr__(self) -> str:
        return f'{type(self).__name__}(state={self.state}, player_1_score={self.player_1_score}, player_2_score={self.player_2_score})'

    def __str__(self) -> str:
        return self.state

    def __copy__(self) -> Self:
        return Termination(
            state=self.state,
            player_1_score=self.player_1_score,
            player_2_score=self.player_2_score
        )

    def __deepcopy__(self, memo) -> Self:
        return self.__copy__()

    def copy(self) -> Self:
        return self.__copy__()