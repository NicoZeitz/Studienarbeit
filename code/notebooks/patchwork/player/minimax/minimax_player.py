from typing import List, Optional
import math
import random

from ..player import Player
from ...action import Action
from ...game import Game, CurrentPlayer
from ...state import State
from ...termination import Termination
from ...time_board import TimeBoard
from ...quilt_board import QuiltBoard
from ...player_state import PlayerState

class MinimaxPlayer(Player):
    """
    A player that uses minimax in combination with alpha-beta pruning to choose an action.
    """

    __slots__ = ("depth", )

    # ================================ attributes ================================

    depth: int

    # ================================ constructor ================================

    def __init__(self, name: Optional[str], depth = 3):
        super().__init__(name=name)
        self.depth = depth

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
        termination = game.get_termination(state)
        if depth == 0 or termination.is_terminated:
            return self.get_static_evaluation_of_state(game, state, termination)

        valid_actions = game.get_valid_actions(state)

        if state.current_active_player == CurrentPlayer.PLAYER_1:
            maxEvaluation = -math.inf
            for action in valid_actions:
                next_state = game.get_next_state(state, action)

                evaluation = self.minimax(next_state, depth - 1, alpha, beta)
                maxEvaluation = max(maxEvaluation, evaluation)
                alpha = max(alpha, evaluation)
                if beta <= alpha:
                    break

            return minEvaluation
        else:
            minEvaluation = math.inf
            for action in valid_actions:
                next_state = game.get_next_state(state, action)

                evaluation = self.minimax(next_state, depth - 1, alpha, beta)
                minEvaluation = max(minEvaluation, evaluation)
                beta = min(beta, evaluation)
                if beta <= alpha:
                    break

            return minEvaluation

    def get_static_evaluation_of_state(self, game: Game, state: State, termination: Termination) -> float:
        if termination.is_terminated:
            if termination.winner == CurrentPlayer.PLAYER_1:
                return math.inf
            elif termination.winner == CurrentPlayer.PLAYER_2:
                return -math.inf
            else:
                return 0

        player_1_score = self.get_static_evaluation_of_player(game, state, CurrentPlayer.PLAYER_1)
        player_2_score = self.get_static_evaluation_of_player(game, state, CurrentPlayer.PLAYER_2)

        return player_1_score - player_2_score

    def get_static_evaluation_of_player(self, game: Game, state: State, player: CurrentPlayer) -> float:

        button_income = self.get_static_evaluation_of_button_income(game, state, player)

        player_position = self.get_static_evaluation_of_position(game, state, player)

        return button_income + player_position

    def get_static_evaluation_of_button_income(self, game: Game, state: State, player: CurrentPlayer) -> float:
        # At the beginning +8 Points für every button income
        # Then exponential decay, at the end only +1 Point for every button income
        # f(x) = 8 * exp(ln(1/8) * x / 8)

        player_position = state.player_1.position if player == CurrentPlayer.PLAYER_1 else state.player_2.position

        amount_button_income_triggers_passed = state.time_board.get_amount_button_income_triggers_in_range(range(0, player_position + 1))

        return 8 * math.exp(math.log(1 / 8) * amount_button_income_triggers_passed / 8)

    def get_static_evaluation_of_position(self, game: Game, state: State, player: CurrentPlayer) -> float:
        player_position = state.player_1.position if player == CurrentPlayer.PLAYER_1 else state.player_2.position
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
