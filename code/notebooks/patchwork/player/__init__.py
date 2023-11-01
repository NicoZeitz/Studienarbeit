__all__ = [
    'HumanPlayer',
    'MCTSPlayer',
    'Player',
    'RandomPlayer',
]

from .player import Player

from .human import HumanPlayer
from .mcts import MCTSPlayer
from .random import RandomPlayer