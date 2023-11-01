from typing import List, Optional
import math
import multiprocessing 

import numpy as np

from .node import Node
from .mcts_options import MCTSOptions

from ..player import Player
from ...action import Action
from ...game import Game
from ...state import State

def mcts_get_action(game: Game, state: State, options: MCTSOptions) -> List[float]:
    root = Node(game, state, options)

    for _ in range(options["number_of_simulations"]):
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

    visit_probabilities = np.array(list(map(lambda child: child.visit_count, root.children)), dtype=np.float64)
    visit_probabilities /= np.sum(visit_probabilities)

    return visit_probabilities

class MCTSPlayer(Player):
    """
    A player that uses Monte Carlo Tree Search to choose an action.
    """

    __slots__ = ('options',)

    # ================================ attributes ================================

    options: MCTSOptions
    """The options for the MCTS algorithm."""

    # ================================ constructor ================================

    def __init__(self, name: Optional[str], options: Optional[MCTSOptions] = None):
        super().__init__(name=name)
        self.options = options if options is not None else {
            'C': math.sqrt(2),
            'number_of_simulations': 10000 # TODO: use higher number (> 1000)
        }

    # ================================ methods ================================

    def get_action(
            self,
            game: Game,
            state: State,
            valid_actions: List[Action]
    ) -> Action:
        if len(valid_actions) == 1:
            return valid_actions[0]

        # pool = multiprocessing.Pool(processes=10)
        # results = np.array(pool.starmap(mcts_get_action, ((game, state, self.options) for _ in range(10))))
        # chosen_action_index = np.argmax(np.sum(results, axis=0))

        # return valid_actions[chosen_action_index]

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

        with open('tree.txt', 'wb') as f:
            f.write(str(root).encode('utf8'))


        raise Exception()

        return root.children[chosen_action_index].action_taken