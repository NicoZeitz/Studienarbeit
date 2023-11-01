from copy import deepcopy
from dataclasses import dataclass
from enum import Enum
from typing import Any, List, Mapping, Optional, Self, Union
import itertools

from .patch import Patch
from .player_state import PlayerState
from .time_board import EntitiesEnum, TimeBoard

class CurrentPlayer(Enum):
    PLAYER_1 = EntitiesEnum.PLAYER_1
    PLAYER_2 = EntitiesEnum.PLAYER_2

@dataclass(slots=True)
class State:
    """
    Represents the full state of the patchwork board game.
    """

    # ================================ attributes ================================

    patches: List[Patch]
    """The patches that are still available to be bought."""

    time_board: TimeBoard
    """The time board of the game."""

    player_1: PlayerState
    """The first player of the game."""

    player_2: PlayerState
    """The second player of the game."""

    current_active_player: CurrentPlayer = CurrentPlayer.PLAYER_1
    """The current player of the game."""

    special_patch_placement_move: Optional[int] = None
    """Whether the current player has to place a special patch as the next move and if so which special patch it is."""

    # ================================ properties ================================

    @property
    def current_player(self) -> PlayerState:
        """Returns the current player."""
        if self.current_active_player == CurrentPlayer.PLAYER_1:
            return self.player_1
        else:
            return self.player_2

    @property
    def other_player(self) -> PlayerState:
        """Returns the other player."""
        if self.current_active_player == CurrentPlayer.PLAYER_1:
            return self.player_2
        else:
            return self.player_1

    # ================================ methods ================================

    def switch_current_player(self) -> None:
        """Switches the current player."""
        if self.current_active_player == CurrentPlayer.PLAYER_1:
            self.current_active_player = CurrentPlayer.PLAYER_2
        else:
            self.current_active_player = CurrentPlayer.PLAYER_1


    def __eq__(self, other: Any) -> Union[NotImplemented, bool]:
        if not isinstance(other, State):
            return NotImplemented

        return self.patches == other.patches and \
            self.time_board == other.time_board and \
            self.player_1 == other.player_1 and \
            self.player_2 == other.player_2 and \
            self.current_active_player == other.current_active_player and \
            self.special_patch_placement_move == other.special_patch_placement_move

    def __hash__(self) -> int:
        return hash((
            self.patches,
            self.time_board,
            self.player_1,
            self.player_2,
            self.current_active_player,
            self.special_patch_placement_move
        ))

    def __repr__(self) -> str:
        return f'{type(self).__name__}(patches={self.patches}, time_board={self.time_board}, player_1={self.player_1}, player_2={self.player_2}, current_player={self.current_active_player}, special_patch_placement_move={self.special_patch_placement_move})'

    def __str__(self) -> str:
        state_str = f'Current player is {self.current_player.name}'
        if self.special_patch_placement_move is not None:
            state_str += f' (special patch placement move {self.special_patch_placement_move + 1})'
        state_str += '\n\n'

        player_1_str = str(self.player_1).split('\n')
        player_2_str = str(self.player_2).split('\n')

        # pad each line in player 1 to the same length
        max_length = max(len(line) for line in player_1_str)
        for index, line in enumerate(player_1_str):
            player_1_str[index] += ' ' * (max_length - len(line))

        divider = (' │ \n' * len(player_1_str)) .split('\n')

        state_str += '\n'.join(' '.join(line) for line in zip(player_1_str, divider, player_2_str))

        state_str += f'\n\nTime board:\n{self.time_board}\n'
        state_str += f'Next 3 patches:\n'

        # only take first 3 patches
        patch_strings = [str(patch).split('\n') for patch in itertools.islice(self.patches, 3)]
        # make each list the same length
        max_length = max(len(patch_string) for patch_string in patch_strings)
        for patch_string in patch_strings:
            patch_string[:0] = ([' '] * (max_length - len(patch_string)))

        # pad each string right to 15 characters
        for patch_string in patch_strings:
            patch_string[:] = [patch_string + ' ' * (15 - len(patch_string)) for patch_string in patch_string]

        # join the strings
        state_str += '\n'.join('   '.join(line) for line in zip(*patch_strings))

        return state_str

    def __copy__(self) -> Self:
        return State(
            patches=self.patches,
            time_board=self.time_board,
            player_1=self.player_1,
            player_2=self.player_2,
            current_active_player=self.current_active_player,
            special_patch_placement_move=self.special_patch_placement_move
        )

    def __deepcopy__(self, memo: Mapping) -> Self:
        return State(
            patches=deepcopy(self.patches, memo),
            time_board=deepcopy(self.time_board, memo),
            player_1=deepcopy(self.player_1, memo),
            player_2=deepcopy(self.player_2, memo),
            current_active_player=self.current_active_player,
            special_patch_placement_move=self.special_patch_placement_move
        )

    def copy(self) -> Self:
        return State(
            patches=list(self.patches),
            time_board=self.time_board.copy(),
            player_1=self.player_1.copy(),
            player_2=self.player_2.copy(),
            current_active_player=self.current_active_player,
            special_patch_placement_move=self.special_patch_placement_move
        )