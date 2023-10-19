import numpy as np

from typing import Union, List
import itertools

from .entities_enum import EntitiesEnum
from .patch import Patch
from .quilt_board import QuiltBoard
from .state import State
from .time_board import TimeBoard

class Game:
    def __init__(self):
        pass

    def get_initial_state(self, seed: Union[int, None] = None):
        """
        1. Each player takes a quilt board, a time token and
           5 buttons (as currency). Keep the remaining butt ons
           on the table close at hand.
        """
        player_1_quilt_board = QuiltBoard()
        player_2_quilt_board = QuiltBoard()
        player_1_button_balance = 5
        player_2_button_balance = 5

        """
        2. Place the central
           time board in the
           middle of the table.
        """

        """
        3. Place your time tokens on
           the starting space of the
           time board.
           The player who last used a
           needle begins
        """
        game_board = TimeBoard()
        current_player = EntitiesEnum.PLAYER_1

        """
        4. Place the (regular)
           patches in a circle or
           oval around the time
           board.
        """

        """
        5. Locate the smallest patch, i.e. the
           patch of size 1x2, and place the
           neutral token between this patch
           and the next patch in clockwise
           order.
        """
        pieces = Patch.generate_pieces(seed=seed)

        """
        6. Lay out the
           special tile
        """

        """
        7. Place the special
           patches on the
           marked spaces of
           the time board
        """


        """
        8. Now you are
           ready to go!
        """
        return State(
            pieces=pieces,
            time_board=game_board,
            player_1_quilt_board=player_1_quilt_board,
            player_2_quilt_board=player_2_quilt_board,
            current_player=current_player,
            player_1_button_balance=player_1_button_balance,
            player_2_button_balance=player_2_button_balance
        )

    def get_next_state(self, state, action, player):
        pass

    def get_valid_moves(self, state: State) -> List[State]:
        # Course of Play
        #
        # In this game, you do not necessarily alternate between turns. The player whose time token is the furthest
        # behind on the time board takes his turn. This may result in a player taking multiple turns in a row before
        # his opponent can take one.
        # If both time tokens are on the same space, the player whose token is on top goes first.
        #
        # On your turn, you carry out one of the following actions:
        valid_moves = []

        # A: Advance and Receive Buttons
        valid_moves.extend(self.get_advance_and_receive_buttons_moves(state))

        # B: Take and Place a Patch
        valid_moves.extend(self.get_take_and_place_a_patch_moves(state))

        return valid_moves

    def get_advance_and_receive_buttons_moves(self, state: State) -> List[State]:
        """
        get the valid moves for the action "Advance and Receive Buttons"

        most of the time this method will return exactly 1 valid move, but if the player walks over a special patch
        there will be multiple valid moves

        :param state: the current state (will not be modified)
        :return: a list of all valid next states
        """

        # Move your time token on the time board so that it occupies the space directly in front of your opponent’s  time token.
        # You receive 1 button (i.e. a butt on tile of value 1) per space you moved your time token

        walking_state = state.copy()
        positions = state.player_positions

        # get a view of the board between the two players (the tiles that the current player will walk over)
        # starting from `current_player_position + 1` because we don't want to include the current player's position
        # ending at `other_player_position + 2` because +1 to include the other players position and another +1 to include the field after the other players position
        walking_view = walking_state.time_board.board[positions.current_player + 1:positions.other_player + 2]

        # 1. add +1 button income for each field the player walks over
        walking_state.add_current_player_button_balance(positions.other_player - positions.current_player + 1)

        # 2. add all button incomes that happens because the player walks over a button income trigger
        button_income_trigger_passed = np.count_nonzero(np.where(walking_view & EntitiesEnum.BUTTON_INCOME_TRIGGER > 0))
        current_player_button_income = walking_state.current_player_button_income
        walking_state.add_current_player_button_balance(button_income_trigger_passed * current_player_button_income)

        # 3. update current player position
        walking_state.set_current_player_position(positions.other_player + 1)

        # 4. switch to the next player for all moves
        walking_state.switch_current_player()

        # 5. add all the possible special patches that the player walks over
        special_patches_indices = np.where(walking_view & EntitiesEnum.SPECIAL_PATCH > 0)
        special_patches_passed = np.count_nonzero(special_patches_indices)
        if(special_patches_passed == 0):
            return [walking_state]

        # FIXME:PERF: Remove for more performance
        assert special_patches_passed <= 1, "Player can only walk over at most one special patch"

        valid_moves = []
        special_piece = Patch.get_special_piece()
        for special_piece_placement in walking_state.current_player_quilt_board.get_valid_patch_placements(special_piece):
            new_state = walking_state.copy()
            new_state.current_player_quilt_board = special_piece_placement
            valid_moves.append(new_state)

        # TODO: Remove
        print(special_patches_indices)

        # 6. Remove the special piece from the pieces
        index = positions.current_player + 1 + np.argmax(special_patches_indices)
        walking_state.time_board.board[index] = walking_state.time_board.board[index] ^ EntitiesEnum.SPECIAL_PATCH

        return valid_moves

    def get_take_and_place_a_patch_moves(self, state: State) -> List[State]:

        for piece in itertools.islice(state.pieces, 3):
            # player can only place pieces that they can afford
            if piece.button_cost > state.current_player_button_balance:
                continue

            new_player_quilt_boards = state.current_player_quilt_board.get_valid_patch_placements(piece)

            # player can only place pieces that fit on their board
            if len(new_player_quilt_boards) == 0:
                continue

            for new_player_quilt_board in new_player_quilt_boards:
                self.get_take_and_place_a_patch_move()
                print('hihih')
                # TODO:
                # new_state = state.copy()
                # new_state.pieces.remove(piece)
                # new_state.switch_current_player()
                # valid_moves.append(new_state)


        return []


    def get_take_and_place_a_patch_move(self, state: State, piece: Patch, new_player_quilt_board: QuiltBoard) -> State:
        pass

    def check_win(self, state, action):
        pass

    def get_value_and_terminated(self, state, action):
        pass