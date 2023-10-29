from typing import Optional
import time

from IPython.display import clear_output
import numpy as np

from .game import Game, ValueAndTerminated

class GameLoop:
    game: int = 0

    def test(self, /, amount: int = 10, sleep: float = 0.5):
        for i in range(0, amount):
            self.run(seed=i, automatic=True, sleep=sleep)
            self.game += 1
            time.sleep(sleep)

    def run(
        self,
        /,
        seed: Optional[int] = None,
        automatic: bool = False,
        sleep: float = 0
    ):
        game = Game()
        state = game.get_initial_state(seed)

        i = 1
        action = 0
        valid_actions = []
        while True:
            try:
                valid_actions = game.get_valid_actions(state)

                clear_output(wait=True)
                print(f"======================= GAME {self.game} TURN {i} =======================")
                print(state)

                # Action -> special patch
                # Action: Walk, Take first, Take second, Take third
                # TODO: Row
                # TODO: Column
                # TODO: Rotation
                # TODO: Flip

                action = 0
                if not automatic:
                    action = int(input(f"Player '{state.current_player.name}' has {len(valid_actions)} options:"))

                if action < 0:
                    action = len(valid_actions) + action + 1
                elif action == 0:
                    action = np.random.randint(1, len(valid_actions) + 1)

                if valid_actions[action - 1] == None:
                    print("action not valid")
                    continue

                print(f"Player '{state.current_player.name}' chose action {action}: {str(valid_actions[action - 1])}")

                state = game.get_next_state(state, valid_actions[action - 1])
                value_and_terminated = game.get_value_and_terminated(state)

                if value_and_terminated.is_terminated:
                    print("======================= GAME ENDED =======================")
                    print(state)
                    print('\n\n')
                    if value_and_terminated == ValueAndTerminated.PLAYER_1_WON:
                        print(f"Player {state.player_1.name} won")
                    elif value_and_terminated == ValueAndTerminated.PLAYER_2_WON:
                        print(f"Player {state.player_2.name} won")
                    else:
                        print("Draw")
                    break
                i+=1
                time.sleep(sleep)
            except Exception as e:
                print("======================= EXCEPTION =======================")
                print(e)
                print("======================= STATE =======================")
                print(state)
                print("======================= ACTION =======================")
                print(valid_actions[action - 1])
                raise