from typing import List, Optional
import random

from ..player import Player
from ...action import Action
from ...game import Game
from ...state import State

class RandomPlayer(Player):
    """
    a player that chooses a random action
    """

    seed: Optional[int]
    """The seed to use for the random number generator."""

    _random: random.Random
    """The random number generator."""

    def __init__(self, name: Optional[str], seed: Optional[int] = None):
        super().__init__(name=name)
        self._random = random.Random(seed)

    def get_action(
            self,
            game: Game,
            state: State,
            valid_actions: List[Action]
    ) -> Action:
        index = self._random.randint(0, len(valid_actions) - 1)
        action = valid_actions[index]
        return action