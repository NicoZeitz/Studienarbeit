from typing import Optional
import time
from timeit import default_timer as timer

from IPython.display import clear_output

from .game import Game, ValueAndTerminated
from .state import CurrentPlayer
from .player import RandomPlayer

class GameLoop:
    game: int = 0

    def test(self, /, amount: int = 10, sleep: float = 0.5):
        for i in range(0, amount):
            self.run(seed=i, sleep=sleep)
            self.game += 1
            time.sleep(sleep)

    def run(
        self,
        /,
        seed: Optional[int] = None,
        sleep: float = 0
    ):
        player_1 = RandomPlayer(name='Player 1 (Random)')
        player_2 = RandomPlayer(name='Player 2 (Random)')

        game = Game()
        state = game.get_initial_state(seed, player_1_name=player_1.name, player_2_name=player_2.name)

        i = 1
        action = None
        valid_actions = []
        avg_get_valid_actions = []
        avg_get_next_state = []
        avg_get_value_and_terminated = []
        avg_get_player_1_action = []
        avg_get_player_2_action = []
        while True:
            try:
                start_time = timer()
                valid_actions = game.get_valid_actions(state)
                avg_get_valid_actions.append(timer() - start_time)

                clear_output(wait=True)
                print(f"======================= GAME {self.game} TURN {i} =======================")
                print(state)

                if state.current_active_player == CurrentPlayer.PLAYER_1:
                    start_time = timer()
                    action = player_1.get_action(game, state, valid_actions)
                    avg_get_player_1_action.append(timer() - start_time)
                else:
                    start_time = timer()
                    action = player_2.get_action(game, state, valid_actions)
                    avg_get_player_2_action.append(timer() - start_time)

                print(f"Player '{state.current_player.name}' chose action: {str(action)}")

                start_time = timer()
                state = game.get_next_state(state, action)
                avg_get_next_state.append(timer() - start_time)

                start_time = timer()
                value_and_terminated = game.get_value_and_terminated(state)
                avg_get_value_and_terminated.append(timer() - start_time)

                if value_and_terminated.is_terminated:
                    clear_output(wait=True)
                    print(f"======================= GAME {self.game} ENDED AFTER {i} TURNS =======================")
                    print(state)
                    print('\n\n')
                    if value_and_terminated == ValueAndTerminated.PLAYER_1_WON:
                        print(f"Player {state.player_1.name} won")
                    elif value_and_terminated == ValueAndTerminated.PLAYER_2_WON:
                        print(f"Player {state.player_2.name} won")
                    else:
                        print("Draw")

                    print(f"Game took {i} turns")
                    print(f"Player '{state.player_1.name}' score: {game.get_score(state, CurrentPlayer.PLAYER_1)}")
                    print(f"Player '{state.player_2.name}' score: {game.get_score(state, CurrentPlayer.PLAYER_2)}")
                    print(f"Average get_valid_actions time: {sum(avg_get_valid_actions) / len(avg_get_valid_actions) * 1000}ms")
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