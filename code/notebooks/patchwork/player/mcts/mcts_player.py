from typing import List, Optional
import math

import numpy as np

from .node import Node
from .mcts_options import MCTSOptions

from ..player import Player
from ...action import Action
from ...game import Game
from ...state import State

class MCTSPlayer(Player):
    """
    A player that uses Monte Carlo Tree Search to choose an action.
    """

    options: MCTSOptions
    """The options for the MCTS algorithm."""

    def __init__(self, name: Optional[str], options: Optional[MCTSOptions] = None):
        super().__init__(name=name)
        self.options = options if options is not None else {
            'C': math.sqrt(2),
            'number_of_simulations': 1000 # TODO: use 1000
        }

    def get_action(
            self,
            game: Game,
            state: State,
            valid_actions: List[Action]
    ) -> Action:
        if len(valid_actions) == 1:
            return valid_actions[0]

        root = Node(game, state, self.options)

        for _ in range(self.options["number_of_simulations"]):
            node = root

            # 1. Selection
            while node.is_fully_expanded() and not node.is_terminal:
                node = node.select()

            # 2. Expansion
            termination = game.get_termination(node.state)

            if not termination.is_terminated:
                # 2. Expansion
                node = node.expand()
                # 3. Simulation
                value = node.simulate()
            else:
                value = termination.score

            # 4. Backpropagation
            node.backpropagate(value)

        chosen_action_index = np.argmax(list(map(lambda child: child.visit_count, root.children)))

        return root.children[chosen_action_index].action_taken