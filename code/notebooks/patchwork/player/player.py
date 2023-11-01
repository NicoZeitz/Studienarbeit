from abc import ABC, abstractmethod
from typing import Any, List, Optional, Unpack

from ..action import Action
from ..game import Game
from ..state import State

class Player(ABC):
    """
    A base class for any kind of player.
    """

    # ================================ attributes ================================

    name: str
    """The name of the player."""

    # ================================ constructor ================================

    def __init__(self, name: Optional[str], **kwargs: Unpack[Any]):
        self.name = name if name is not None else self.__class__.__name__

    # ================================ abstract methods ================================

    @abstractmethod
    def get_action(
        self,
        game: Game,
        state: State
    ) -> Action:
        """
        A method that returns an action given the current state of the game and the valid actions.

        :param game: The game.
        :param state: The state of the game.
        :return: The action to take.
        """
        pass