from abc import ABC, abstractmethod
from typing import List, Optional

from ..action import Action
from ..game import Game
from ..state import State

class Player(ABC):
    """
    A base class for any kind of player.
    """

    name: str
    """The name of the player."""

    def __init__(self, name: Optional[str], **kwargs: dict):
        self.name = name if name is not None else self.__class__.__name__

    @abstractmethod
    def get_action(
        self,
        game: Game,
        state: State,
        valid_actions: List[Action]
    ) -> Action:
        """
        A method that returns an action given the current state of the game and the valid actions.

        :param game: The game.
        :param state: The state of the game.
        :param valid_actions: The valid actions.
        :return: The action to take.
        """
        pass