from typing import Self, Optional, List, Generator
import math
import random
from functools import cached_property

import numpy as np

from .mcts_options import MCTSOptions

from ...termination import Termination
from ...action import Action
from ...game import Game, CurrentPlayer
from ...state import State

class Node:
    """
    A node in the Monte Carlo Tree Search algorithm.
    """

    __slots__ = ('game', 'state', 'options', 'parent', 'action_taken', 'children', 'score_sum', 'max_score', 'min_score', 'visit_count', '__dict__')

    # ================================ attributes ================================

    game: Game
    """The game."""

    state: State
    """The state of the game."""

    options: MCTSOptions
    """The options for the MCTS algorithm."""

    parent: Optional[Self]
    """The parent node. None if this is the root node."""

    action_taken: Optional[Action]
    """The action that was taken to get to this node. None if this is the root node."""

    children: List[Self]
    """The children nodes."""

    score_sum: int
    """The sum of the scores of all the nodes in the subtree rooted at this node."""

    max_score: int
    """The maximum score of all the nodes in the subtree rooted at this node."""

    min_score: int
    """The minimum score of all the nodes in the subtree rooted at this node."""

    visit_count: int
    """The number of times this node has been visited."""

    # ================================ properties ================================

    @cached_property
    def expandable_actions(self) -> List[Action]:
        """
        The moves that can be expanded.
        """
        actions = self.game.get_valid_actions(self.state)
        random.shuffle(actions)
        return actions

    @property
    def is_terminal(self) -> bool:
        """
        Checks whether this node is a terminal node.
        A node is a terminal node if the game is over.
        """
        return self.termination.is_terminated

    @cached_property
    def termination(self) -> Termination:
        """
        Gets the termination of the game from the view of the node.
        """
        return self.game.get_termination(self.state)

    # ================================ constructor ================================

    def __init__(self, game: Game, state: State, options: MCTSOptions, parent: Optional[Self] = None, action_taken: Optional[Action] = None):
        self.game = game
        self.state = state
        self.options = options
        self.parent = parent
        self.action_taken = action_taken

        self.children = []

        self.score_sum = 0
        self.visit_count = 0
        self.max_score = -math.inf
        self.min_score = math.inf

    # ================================ methods ================================

    def is_fully_expanded(self) -> bool:
        """
        Checks whether this node is fully expanded.
        A node is fully expanded if all its children are expanded or if it is a terminal node.
        """
        return self.is_terminal or len(self.expandable_actions) == 0

    def select(self) -> Self:
        """
        Selects the child node with the highest upper confidence bound.

        :return: The child node with the highest upper confidence bound.
        """

        index = np.argmax(map(lambda child: self.get_upper_confidence_bound(child), self.children))

        return self.children[index]

    def get_upper_confidence_bound(self, child: Self) -> float:
        """
        Calculates the upper confidence bound of this node.
        Formula: Q(s, a) + C * sqrt(ln(N(s)) / N(s, a))
        Where:
            Q(s, a) is the average value of the node
            C is the exploration parameter
            N(s) is the visit count of the node
            N(s, a) is the visit count of the child node

        :param child: The child node.
        :return: The upper confidence bound of this node.
        """

        # TODO: Test if it is working correctly

        # https://stackoverflow.com/questions/36664993/mcts-uct-with-a-scoring-system
        # There is an alternative way of setting C dynamically that has given me good results. As you play, you keep track of the highest and lowest scores you've ever seen in each node (and subtree). This is the range of scores possible and this gives you a hint of how big C should be in order to give not well explored underdog nodes a fair chance. Every time i descend into the tree and pick a new root i adjust C to be sqrt(2) * score range for the new root. In addition, as rollouts complete and their scores turn out the be a new highest or lowest score i adjust C in the same way. By continually adjusting C this way as you play but also as you pick a new root you keep C as large as it needs to be to converge but as small as it can be to converge fast. Note that the minimum score is as important as the max: if every rollout will yield at minimum a certain score then C won't need to overcome it. Only the difference between max and min matters.

        score_range = self.max_score - self.min_score
        exploitation = child.score_sum / child.visit_count
        exploration = self.options["C"] * score_range * math.sqrt(math.log(self.visit_count) / child.visit_count)

        return exploitation + exploration

    def expand(self) -> Self:
        """
        Expands this node by adding a child node.
        The child node is chosen randomly from the expandable actions.
        """
        action = self.expandable_actions.pop()

        child_state = self.state.copy()
        child_state = self.game.get_next_state(child_state, action)

        child = Node(self.game, child_state, self.options, parent=self, action_taken=action)
        self.children.append(child)
        return child

    def simulate(self) -> float:
        """
        Simulates a game from this node until the end.

        :return: The value of the game.
        """
        termination = self.termination
        if termination.is_terminated:
            multiplier = 1 if self.state.current_active_player == CurrentPlayer.PLAYER_1 else -1
            return multiplier * termination.score

        rollout_state = self.state.copy()
        valid_actions = self.expandable_actions
        action = random.choice(valid_actions)

        while True:
            try:
                rollout_state = self.game.get_next_state(rollout_state, action)
                termination = self.game.get_termination(rollout_state)
            except Exception as e:
                print(e)
                print(action)
                raise e

            if termination.is_terminated:
                multiplier = 1 if rollout_state.current_active_player == CurrentPlayer.PLAYER_1 else -1
                return multiplier * termination.score

            action = self.game.sample_random_action(rollout_state)

    def backpropagate(self, score: int) -> None:
        """
        Backpropagates the score of the game to the parent nodes.

        :param score: The score at the end of the game that should be backpropagated.
        """
        self.score_sum += score
        self.visit_count += 1

        if score > self.max_score:
            self.max_score = score
        if score < self.min_score:
            self.min_score = score

        if self.parent is not None:
            if self.parent.state.current_active_player != self.state.current_active_player:
                score = -score

            self.parent.backpropagate(score)

    def __repr__(self) -> str:
        return f'{type(self)}(player={self.state.current_player.name}, score_sum={self.score_sum}, visit_count={self.visit_count}, action_taken={self.action_taken}, max_score={self.max_score}, min_score={self.min_score}, ucb={self.parent.get_upper_confidence_bound(self) if self.parent is not None else 0})'

    def tree_list(self) -> Generator[str, None, None]:
        """
        Yields a list of strings that represent the tree.
        """
        yield self.__repr__()

        for index, child in enumerate(self.children):
            branching_front = '├── ' if index != len(self.children) - 1 else '└── '
            other_front = '│   ' if index != len(self.children) - 1 else '    '

            for inner_index, line in enumerate(child.tree_list()):
                front = branching_front if inner_index == 0 else other_front
                yield f'{front}{line}'

    def __str__(self) -> str:
        return '\n'.join(self.tree_list())