from enum import Enum
from typing import Union, List, Optional, Self
import itertools

import numpy as np

from .patch import Patch
from .player_state import PlayerState
from .quilt_board import QuiltBoard
from .state import State, CurrentPlayer
from .time_board import TimeBoard
from .action import Action

class ValueAndTerminated(Enum):
    PLAYER_1_WON = 1
    PLAYER_2_WON = -1
    DRAW = 0
    NOT_TERMINATED = 2

    @property
    def is_terminated(self: Self) -> bool:
        return self != ValueAndTerminated.NOT_TERMINATED

class Game:
    """
    The game of Patchwork.
    """

    # ================================ instance methods ================================

    def get_initial_state(
            self,
            /,
            seed: Optional[int] = None,
            player_1_name: Optional[str] = None,
            player_2_name: Optional[str] = None
    ) -> State:
        """
        Gets the initial state of the game.

        :param seed: The seed to use for the random number generator.
        :param player_1_name: The name of the first player.
        :param player_2_name: The name of the second player.
        :return: The initial state of the game.
        """

        # 1. Each player takes a quilt board, a time token and 5 buttons
        #    (as currency). Keep the remaining buttons on the table close at
        #    hand.
        player_1 = PlayerState(
            name='Player 1' if player_1_name is None else player_1_name,
            position=0,
            button_balance=5,
            quilt_board=QuiltBoard()
        )
        player_2 = PlayerState(
            name='Player 2' if player_2_name is None else player_2_name,
            position=0,
            button_balance=5,
            quilt_board=QuiltBoard()
        )

        # 2. Place the central time board in the middle of the table.

        # 3. Place your time tokens on the starting space of the
        #    time board. The player who last used a needle begins
        game_board = TimeBoard()
        current_active_player = CurrentPlayer.PLAYER_1

        # 4. Place the (regular) patches in a circle or oval around the time
        #    board.

        # 5. Locate the smallest patch, i.e. the patch of size 1x2, and place
        #    the neutral token between this patch and the next patch in
        #    clockwise order.
        patches = Patch.generate_patches(seed=seed)

        # 6. Lay out the special tile

        # 7. Place the special patches on the marked spaces of the time board

        # 8. Now you are ready to go!
        return State(
            patches=patches,
            time_board=game_board,
            player_1=player_1,
            player_2=player_2,
            current_active_player=current_active_player,
        )

    def get_valid_actions(self, state: State) -> List[Action]:
        """
        Gets the valid actions for the current player in the given state.

        :param state: The state of the game.
        :return: The valid actions for the current player in the given state.
        """

        # Course of Play
        #
        # In this game, you do not necessarily alternate between turns. The
        # player whose time token is the furthest behind on the time board takes
        # his turn. This may result in a player taking multiple turns in a row
        # before his opponent can take one.
        # If both time tokens are on the same space, the player whose token is
        # on top goes first.

        # Placing a Special Patch is a special action
        if state.special_patch_placement_move is not None:
            special_patch = Patch.get_special_patch(state.special_patch_placement_move)
            return state.current_player.quilt_board.get_valid_actions_for_patch(special_patch)

        # On your turn, you carry out one of the following actions:
        valid_actions: List[Action] = []

        # A: Advance and Receive Buttons
        valid_actions.extend(self._get_advance_and_receive_buttons_actions(state))

        # B: Take and Place a Patch
        valid_actions.extend(self._get_take_and_place_a_patch_actions(state))

        return valid_actions

    def get_next_state(self, state: State, action: Action) -> State:
        """
        Gets the next state of the game after the given action has been taken.

        :param state: The state of the game.
        :param action: The action to take.
        :return: The next state of the game.
        """

        new_state = state.copy()

        # IF special patch
        #   1. place patch
        #   2. switch player
        #   3. reset special patch state
        if new_state.special_patch_placement_move:
            new_state.current_player.quilt_board.add_patch(action.patch, action.patch_position)
            new_state.switch_current_player()
            new_state.special_patch_placement_move = None
            return new_state

        old_current_player_position = new_state.current_player.position
        other_player_position = new_state.other_player.position
        time_cost: int = 0

        # IF walking
        #   1. add +1 to current player button balance for every tile walked over
        if action.is_walking:
            time_cost = other_player_position - old_current_player_position + 1
            new_state.current_player.button_balance += time_cost

        # IF patch placement
        #  1. place patch
        #  2. remove patch from available patches
        #  3. subtract button cost from current player button balance
        #      a) if the board is full the current player get +7 points
        elif action.is_patch_placement:
            new_state.current_player.quilt_board.add_patch(action.patch, action.patch_position)
            new_state.patches = np.roll(new_state.patches, -action.patch_index-1)
            new_state.current_player.button_balance -= action.patch.button_cost
            time_cost = action.patch.time_cost

            if new_state.current_player.quilt_board.is_full:
                new_state.current_player.button_balance += 7

        # 4. move player by time_cost
        new_state.current_player.position += time_cost
        new_current_player_position = new_state.current_player.position
        new_state.time_board.set_player_position(new_state.current_active_player.value, old_current_player_position, new_current_player_position)

        walking_range = range(old_current_player_position + 1, new_current_player_position + 1)

        # 5. test if player moved over button income trigger (multiple possible) and add button income
        button_income_triggers = new_state.time_board.get_amount_button_income_triggers_in_range(walking_range)
        button_income = new_state.current_player.quilt_board.button_income
        new_state.current_player.button_balance += button_income_triggers * button_income

        # 6. test if player moved over special patch (only a single one possible) and conditionally change the state
        special_patches = new_state.time_board.get_special_patches_in_range(walking_range)
        if special_patches.size != 0:
            special_patch_index = special_patches[0]
            new_state.time_board.clear_special_patch(special_patch_index)

            # Test if special patch can even be placed
            if new_state.current_player.quilt_board.is_full:
                # If not throw the special patch away and switch player
                new_state.switch_current_player()
                return new_state

            new_state.special_patch_placement_move = special_patch_index
            return new_state

        # test player position and optionally switch (always true if action.is_walking)
        if new_current_player_position > other_player_position:
            new_state.switch_current_player()

        return new_state

    def get_score(self, state: State, player: CurrentPlayer) -> int:
        if player == CurrentPlayer.PLAYER_1:
            return state.player_1.quilt_board.score + state.player_1.button_balance
        else:
            return state.player_2.quilt_board.score + state.player_2.button_balance

    def get_value_and_terminated(self, state: State) -> ValueAndTerminated:
        """
        Returns if the game is terminated and if so who won.

        :param state: The state of the game.
        :return: The value of the game and whether the game is terminated.
        """

        player_1_position = state.player_1.position
        player_2_position = state.player_2.position

        if player_1_position < TimeBoard.MAX_POSITION or player_2_position < TimeBoard.MAX_POSITION:
            return ValueAndTerminated.NOT_TERMINATED

        player_1_score = self.get_score(state, CurrentPlayer.PLAYER_1)
        player_2_score = self.get_score(state, CurrentPlayer.PLAYER_2)

        if player_1_score > player_2_score:
            return ValueAndTerminated.PLAYER_1_WON
        elif player_1_score < player_2_score:
            return ValueAndTerminated.PLAYER_2_WON
        else:
            return ValueAndTerminated.DRAW

    # ================================ private methods ================================

    def _get_advance_and_receive_buttons_actions(self, state: State) -> List[Action]:
        """
        Get the valid moves for the action "Advance and Receive Buttons"

        :param state: the current state (will not be modified)
        :return: a list with one entry containing the walking action
        """
        return [Action.walking()]

    def _get_take_and_place_a_patch_actions(self, state: State) -> List[Action]:
        """
        Get the valid moves for the action "Take and Place a Patch"

        :param state: the current state (will not be modified)
        :return: a list of all valid next states
        """

        valid_actions: List[Action] = []

        for index, patch in enumerate(itertools.islice(state.patches, 3)):
            # player can only place pieces that they can afford
            if patch.button_cost > state.current_player.button_balance:
                continue

            valid_actions.extend(state.current_player.quilt_board.get_valid_actions_for_patch(patch, index))

        return valid_actions
