from typing import Optional
from timeit import default_timer as timer
import time
import sys

from IPython.display import clear_output, display_html

from .game import Game
from .player import Player
from .termination import Termination
from .state import CurrentPlayer
from .player import RandomPlayer

class GameLoop:

    @staticmethod
    def test(
        iterations: int = 100,
        *,
        player_1: Player,
        player_2: Player,
        seed: Optional[int] = None,
    ):
        """
        Runs the game loop multiple times and records the outcome.

        :param iterations: The number of iterations to run.
        :param player_1: The first player.
        :param player_2: The second player.
        """

        game = Game()

        player_1_won = 0
        player_2_won = 0
        draw = 0
        player_1_scores = []
        player_2_scores = []

        for i in range(iterations):
            state = game.get_initial_state(seed=seed, player_1_name=player_1.name, player_2_name=player_2.name)
            action = None

            clear_output(wait=True)
            print(f"======================= GAME {i} =======================", flush=True)

            while True:
                if state.current_active_player == CurrentPlayer.PLAYER_1:
                    action = player_1.get_action(game, state)
                else:
                    action = player_2.get_action(game, state)

                state = game.get_next_state(state, action)
                termination = game.get_termination(state)

                if termination.is_terminated:
                    if termination == Termination.PLAYER_1_WON:
                        player_1_won += 1
                    elif termination == Termination.PLAYER_2_WON:
                        player_2_won += 1
                    else:
                        draw += 1

                    player_1_scores.append(termination.player_1_score)
                    player_2_scores.append(termination.player_2_score)
                    break

        clear_output(wait=True)
        print(f"Player '{state.player_1.name}' won {player_1_won}/{iterations} times")
        print(f"Player '{state.player_2.name}' won {player_2_won}/{iterations} times")
        print(f"Draw {draw}/{iterations} times")
        print(f"Player '{state.player_1.name}' average score: {sum(player_1_scores) / len(player_1_scores)}")
        print(f"Player '{state.player_2.name}' average score: {sum(player_2_scores) / len(player_2_scores)}")

    @staticmethod
    def test_no_output():
        max_player_1_score = -float('inf')
        max_player_2_score = -float('inf')
        min_player_1_score = float('inf')
        min_player_2_score = float('inf')
        player_1_wins = 0
        player_2_wins = 0
        draws = 0

        for _ in range(1000):
            player_1 = RandomPlayer(name='Player 1 (Random)')
            player_2 = RandomPlayer(name='Player 2 (Random)')
            game = Game()
            state = game.get_initial_state(player_1_name=player_1.name, player_2_name=player_2.name)
            while True:
                if state.current_active_player == CurrentPlayer.PLAYER_1:
                    action = player_1.get_action(game, state)
                else:
                    action = player_2.get_action(game, state)

                state = game.get_next_state(state, action)
                termination = game.get_termination(state)

                if termination.is_terminated:
                    if termination == Termination.PLAYER_1_WON:
                        player_1_wins += 1
                    elif termination == Termination.PLAYER_2_WON:
                        player_2_wins += 1
                    else:
                        draws += 1

                    max_player_1_score = max(max_player_1_score, termination.player_1_score)
                    max_player_2_score = max(max_player_2_score, termination.player_2_score)
                    min_player_1_score = min(min_player_1_score, termination.player_1_score)
                    min_player_2_score = min(min_player_2_score, termination.player_2_score)
                    break

        print(f"max_player_1_score: {max_player_1_score}")
        print(f"max_player_2_score: {max_player_2_score}")
        print(f"min_player_1_score: {min_player_1_score}")
        print(f"min_player_2_score: {min_player_2_score}")
        print(f"player_1_wins: {player_1_wins}")
        print(f"player_2_wins: {player_2_wins}")
        print(f"draws: {draws}")


    @staticmethod
    def run(
        *,
        seed: Optional[int] = None,
        sleep: float = 0,
        player_1: Optional[Player] = None,
        player_2: Optional[Player] = None
    ):
        """
        Runs an interactive game of Patchwork.

        :param seed: The seed to use for the random number generator to place the patches.
        :param sleep: The number of seconds to wait between each turn.
        :param player_1: The first player.
        :param player_2: The second player.
        """

        if player_1 is None:
            player_1 = RandomPlayer(name='Player 1 (Random)')

        if player_2 is None:
            player_2 = RandomPlayer(name='Player 2 (Random)')

        game = Game()
        state = game.get_initial_state(seed=seed, player_1_name=player_1.name, player_2_name=player_2.name)

        i = 1
        action = None
        avg_get_next_state = []
        avg_get_value_and_terminated = []
        avg_get_player_1_action = []
        avg_get_player_2_action = []

        while True:
            try:
                clear_output(wait=True)

                display_html(f"<style></style>", raw=True) # force vscode to render output before input box appears
                print(f"======================= TURN {i} =======================")
                print(state, flush=True)

                if state.current_active_player == CurrentPlayer.PLAYER_1:
                    start_time = timer()
                    action = player_1.get_action(game, state)
                    avg_get_player_1_action.append(timer() - start_time)
                else:
                    start_time = timer()
                    action = player_2.get_action(game, state)
                    avg_get_player_2_action.append(timer() - start_time)

                print(f"Player '{state.current_player.name}' chose action: {str(action)}")

                start_time = timer()
                state = game.get_next_state(state, action)
                avg_get_next_state.append(timer() - start_time)

                start_time = timer()
                termination = game.get_termination(state)
                avg_get_value_and_terminated.append(timer() - start_time)

                if termination.is_terminated:
                    clear_output(wait=True)
                    print(f"======================= GAME ENDED AFTER {i} TURNS =======================")
                    print(state)
                    print('\n\n')
                    if termination == Termination.PLAYER_1_WON:
                        print(f"Player {state.player_1.name} won")
                    elif termination == Termination.PLAYER_2_WON:
                        print(f"Player {state.player_2.name} won")
                    else:
                        print("Draw")

                    print(f"Game took {i} turns")
                    print(f"Player '{state.player_1.name}' score: {termination.player_1_score}")
                    print(f"Player '{state.player_2.name}' score: {termination.player_2_score}")
                    print(f"Average get_next_state time: {sum(avg_get_next_state) / len(avg_get_next_state) * 1000}ms")
                    print(f"Average get_value_and_terminated time: {sum(avg_get_value_and_terminated) / len(avg_get_value_and_terminated) * 1000}ms")
                    print(f"Average get_player_1_action time: {sum(avg_get_player_1_action) / len(avg_get_player_1_action) * 1000}ms")
                    print(f"Average get_player_2_action time: {sum(avg_get_player_2_action) / len(avg_get_player_2_action) * 1000}ms")
                    break
                i+=1
                time.sleep(sleep)
            except Exception as e:
                print("======================= EXCEPTION =======================")
                print(e)
                print("======================= STATE =======================")
                print(state)
                print("======================= ACTION =======================")
                print(action)
                raise