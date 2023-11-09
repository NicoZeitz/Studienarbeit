from typing import List, Optional, NamedTuple
import math
import random

import numpy as np

from ..player import Player
from ...action import Action
from ...game import Game, CurrentPlayer
from ...state import State
from ...termination import Termination
from ...time_board import TimeBoard
from ...quilt_board import QuiltBoard
from ...player_state import PlayerState

class ActionWithScore(NamedTuple):
    """
    A named tuple that represents an action with a score.
    """

    action: Action
    """The action."""

    score: int
    """The score."""

class MinimaxPlayer(Player):
    """
    A player that uses minimax in combination with alpha-beta pruning to choose an action.
    """

    __slots__ = ("depth", )

    # ================================ attributes ================================

    depth: int
    """The depth of the minimax algorithm."""

    amount_of_actions_per_patch: int
    """The amount of actions per patch."""

    # ================================ static attributes ================================

    moore_mask = np.array([
        [1,1,1],
        [1,1,1],
        [1,1,1]
    ], dtype=bool)

    # ================================ constructor ================================

    def __init__(self, name: Optional[str], depth = 2, amount_of_actions_per_patch = 3):
        super().__init__(name=name)
        self.depth = depth
        self.amount_of_actions_per_patch = amount_of_actions_per_patch

    # ================================ methods ================================

    def get_action(
            self,
            game: Game,
            state: State
    ) -> Action:
        valid_actions = game.get_valid_actions(state)

        if len(valid_actions) == 1:
            return valid_actions[0]

        chosen_action = valid_actions[0]
        chosen_score = -math.inf if state.current_active_player == CurrentPlayer.PLAYER_1 else math.inf

        for action in valid_actions:
            next_state = game.get_next_state(state, action)
            player_is_maximizer = state.current_active_player == CurrentPlayer.PLAYER_1
            alpha = -math.inf if player_is_maximizer else math.inf
            beta = -alpha

            score = self.minimax(game, next_state, self.depth-1, alpha, beta)

            if state.current_active_player == CurrentPlayer.PLAYER_1:
                if score > chosen_score:
                    chosen_action = action
                    chosen_score = score
            else:
                if score < chosen_score:
                    chosen_action = action
                    chosen_score = score

            if score == chosen_score:
                chosen_action = random.choice([chosen_action, action])

        return chosen_action

    def minimax(self, game: Game, state: State, depth: int, alpha: float, beta: float):
        """
        Returns the minimax value of the given state.

        :param game: The game.
        :param state: The state.
        :param depth: The depth.
        :param alpha: The alpha value.
        :param beta: The beta value.
        """

        termination = game.get_termination(state)
        if depth == 0 or termination.is_terminated:
            return self.get_static_evaluation_of_state(game, state, termination)

        valid_actions = game.get_valid_actions(state)

        best_actions = self.get_best_actions(game, state, valid_actions)

        if state.current_active_player == CurrentPlayer.PLAYER_1:
            maxEvaluation = -math.inf
            for action in best_actions:
                next_state = game.get_next_state(state, action)

                evaluation = self.minimax(game, next_state, depth - 1, alpha, beta)
                maxEvaluation = max(maxEvaluation, evaluation)
                alpha = max(alpha, evaluation)
                if beta <= alpha:
                    break

            return minEvaluation
        else:
            minEvaluation = math.inf
            for action in best_actions:
                next_state = game.get_next_state(state, action)

                evaluation = self.minimax(game, next_state, depth - 1, alpha, beta)
                minEvaluation = max(minEvaluation, evaluation)
                beta = min(beta, evaluation)
                if beta <= alpha:
                    break

            return minEvaluation
        
    def get_best_actions(self, game: Game, state: State, actions: List[Action]) -> List[Action]:
        """
        
        """

        best_actions: List[Action] = []
        place_first_patch_actions: List[Action] = []
        place_first_patch_scores: List[int] = []
        place_second_patch_actions: List[ActionWithScore] = []
        place_second_patch_scores: List[int] = []
        place_third_patch_actions: List[ActionWithScore] = []
        place_third_patch_scores: List[int] = []
        quilt_board = state.current_player.quilt_board

        for action in actions:
            if action.is_walking:
                best_actions.append(action)
                continue
            
            if action.is_first_patch_taken:
                score = self.get_score_for_patch_placement(quilt_board, action)
                place_first_patch_actions.append(action)
                place_first_patch_scores.append(score)
            
            if action.is_second_patch_taken:
                score = self.get_score_for_patch_placement(quilt_board, action)
                place_second_patch_actions.append(action)
                place_second_patch_scores.append(score)
            
            if action.is_third_patch_taken:
                score = self.get_score_for_patch_placement(quilt_board, action)
                place_third_patch_actions.append(action)
                place_third_patch_scores.append(score)

            if action.is_special_patch_placement:
                score = self.get_score_for_patch_placement(quilt_board, action)
                place_first_patch_actions.append(action)
                place_first_patch_scores.append(score)

        place_first_patch_indices = np.argpartition(place_first_patch_scores, -self.amount_of_actions_per_patch)[-self.amount_of_actions_per_patch:]
        place_second_patch_indices = np.argpartition(place_first_patch_scores, -self.amount_of_actions_per_patch)[-self.amount_of_actions_per_patch:]
        place_third_patch_indices = np.argpartition(place_first_patch_scores, -self.amount_of_actions_per_patch)[-self.amount_of_actions_per_patch:]

        best_actions.extend(np.array(place_first_patch_actions)[place_first_patch_indices])
        best_actions.extend(np.array(place_second_patch_actions)[place_second_patch_indices])
        best_actions.extend(np.array(place_third_patch_actions)[place_third_patch_indices])
        
        return best_actions

    def get_score_for_patch_placement(self, quilt_board: QuiltBoard, action: Action) -> int:
        """
        Returns the score for the given action.

        :param game: The game.
        :param state: The state.
        :param action: The action.
        """
        patch = action.patch
        patch_position = action.patch_position

        quilt_board_tiles = np.pad(quilt_board.tiles.copy(), (1,1), 'constant', constant_values=True)

        quilt_board_tiles[
            patch_position.row    : patch_position.row    + patch.shape[0], 
            patch_position.column : patch_position.column + patch.shape[1]
        ] |= patch.tiles

        score = 0

        for (row, column) in np.ndindex(patch.shape):
            if not patch.tiles[row, column]:
                continue

            window = quilt_board_tiles[
                patch_position.row    + row    : patch_position.row    + row    + 3,
                patch_position.column + column : patch_position.column + column + 3,
            ]

            score -= np.sum(np.bitwise_xor(window, MinimaxPlayer.moore_mask))

        return score


    def get_static_evaluation_of_state(self, game: Game, state: State, termination: Termination) -> float:
        if termination.is_terminated:
            if termination.winner == CurrentPlayer.PLAYER_1:
                return math.inf
            elif termination.winner == CurrentPlayer.PLAYER_2:
                return -math.inf
            else:
                return 0

        player_1_score = self.get_static_evaluation_of_player(state, CurrentPlayer.PLAYER_1)
        player_2_score = self.get_static_evaluation_of_player(state, CurrentPlayer.PLAYER_2)

        return player_1_score - player_2_score

    def get_static_evaluation_of_player(self, state: State, player: CurrentPlayer) -> float:
        """
        Returns the static evaluation of the given player at the current state. The score is offset to always be positive.

        :param state: The state.
        :param player: The player.
        :return: The static evaluation.
        """

        player_state = state.player_1 if player == CurrentPlayer.PLAYER_1 else state.player_2
        player_position = player_state.position

        button_income = self.get_static_evaluation_of_button_income(state, player_position)
        player_position = self.get_static_evaluation_of_position(player_position)
        game_evaluation = self.get_static_evaluation_of_game_end(player_state)

        return button_income + player_position + game_evaluation

    def get_static_evaluation_of_button_income(self, state: State, player_position: int) -> float:
        """
        Returns the static evaluation of the button income from the view of the given player.

        +8 Points für every button income at the beginning
        Then exponential decay, at the end only +1 Point for every button income

        ```
        f(x) = 8 * exp(ln(1/8) * x / 8)
        ```

        :param state: The state.
        :param player: The player.
        :return: The static evaluation.
        """

        amount_button_income_triggers_passed = state.time_board.get_amount_button_income_triggers_in_range(range(0, player_position + 1))

        return 8 * math.exp(math.log(1 / 8) * amount_button_income_triggers_passed / 8)
    
    def get_static_evaluation_of_position(self, player_position: int) -> float:
        """
        Returns the static evaluation of the position from the view of the given player.

        +1 Point for every position not yet passed

        :param player_position: The position.
        :return: The static evaluation.
        """

        return TimeBoard.MAX_POSITION - player_position

    def get_static_evaluation_of_game_end(self, player: PlayerState) -> float:
        """
        Returns the static evaluation of the game at the current state with the same criterion as the game end evaluation from the view of the given player. The score is offset to always be positive.

        +1 Point for every button in button balance
        -2 Points for every free space on the quilt board

        :param player: The player.
        :return: The static evaluation.
        """

        quilt_board = player.quilt_board.score
        button_balance = player.button_balance

        offset_to_positive = 2 * QuiltBoard.TILES

        return quilt_board + button_balance + offset_to_positive
