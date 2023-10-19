import numpy as np
from typing import List, Self

from .patch import Patch

class QuiltBoard:
    def __init__(self):
        self.board = np.zeros((9,9), dtype=bool)
        self.button_income = 0

    def copy(self) -> Self:
        new_patchwork_player_board = QuiltBoard()
        new_patchwork_player_board.board = self.board.copy()
        new_patchwork_player_board.button_income = self.button_income
        return new_patchwork_player_board

    def get_valid_patch_placements(self, patch: Patch) -> List[Self]:
        valid_patch_placements = []
        symmetries = patch.get_unique_symmetries()

        for symmetry in symmetries:
            (symmetry_row, symmetry_column) = symmetry.shape
            for row in range(np.size(self.board, 0) - symmetry_row + 1):
                for column in range(np.size(self.board, 1) - symmetry_column + 1):
                    board_view = self.board[row:row+symmetry_row, column:column+symmetry_column]
                    combination = (symmetry | board_view)

                    ones_in_patch = np.count_nonzero(symmetry == 1)
                    ones_in_view = np.count_nonzero(board_view == 1)
                    ones_in_combination = np.count_nonzero(combination == 1)
                    if ones_in_combination == ones_in_view + ones_in_patch:
                        new_player_board_state = self.copy()
                        new_player_board_state.board[row:row+symmetry_row, column:column+symmetry_column] = combination
                        valid_patch_placements.append(new_player_board_state)

        return valid_patch_placements