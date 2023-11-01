from typing import List
import random

from ...action import Action
from ...game import Game
from ...player import Player
from ...state import State

class HumanPlayer(Player):
    """
    A player that is a human.
    """

    def get_action(
            self,
            game: Game,
            state: State,
            valid_actions: List[Action]
    ) -> Action:
        # TODO: better interactivity
        # REPEAT UNTIL VALID Action
        #    IF Action == special patch:
        #       1. Ask for row
        #       2. Ask for column
        #    ELSE
        #       1. Ask for Walk, Take first, Take second, Take third (only show possible pieces)
        #       IF Walk:
        #          Return Action
        #       ELSE:
        #          1. Ask for row
        #          2. Ask for column
        #          3. Ask for rotation
        #          4. Ask for flip

        prompt = f"Player '{state.current_player.name}' has {len(valid_actions)} options:"

        while True:
            try:
                index = int(input(prompt))

                # begin from end
                if index < 0:
                    index = len(valid_actions) + index + 1

                if index == 0:
                    index = random.randint(1, len(valid_actions) + 1)

                index = index - 1

                if valid_actions[index] != None:
                    return valid_actions[index]

            except (ValueError, IndexError) as e:
                print("Please enter a number valid number in the range.")
                # raise e