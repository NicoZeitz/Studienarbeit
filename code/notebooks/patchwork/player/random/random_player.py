from typing import List, Optional
import random

from ..player import Player
from ...action import Action
from ...game import Game
from ...state import State

class RandomPlayer(Player):
    """
    A player that chooses a random action.
    """

    # ================================ attributes ================================

    seed: Optional[int]
    """The seed to use for the random number generator."""

    _random: random.Random
    """The random number generator."""

    # ================================ constructor ================================

    def __init__(self, name: Optional[str], seed: Optional[int] = None):
        super().__init__(name=name)
        self._random = random.Random(seed)

    # ================================ methods ================================

    def get_action(
            self,
            game: Game,
            state: State
    ) -> Action:
        valid_actions = game.get_valid_actions(state)
        index = self._random.randint(0, len(valid_actions) - 1)
        action = valid_actions[index]
        return action