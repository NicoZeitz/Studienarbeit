from typing import List, Optional
import random

from ...action import Action
from ...game import Game
from ...player import Player
from ...state import State

class HumanPlayer(Player):
    """
    a player that is a human
    """

    def __init__(self, name: Optional[str]):
        super().__init__(name=name)

    def get_action(
            self,
            game: Game,
            state: State,
            valid_actions: List[Action]
    ) -> Action:
        # TODO: better interactivity
        # Action -> special patch
        # Action: Walk, Take first, Take second, Take third
        # Row
        # Column
        # Rotation
        # Flip
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
            except (ValueError, IndexError):
                print("Please enter a number valid number in the range.")
