from collections import namedtuple
from copy import deepcopy
from dataclasses import dataclass
from typing import Literal, Optional, Self

from .patch import Patch

Position = namedtuple('Position', 'row column')

@dataclass
class Action:
    """
    Represents an action that can be taken in the patchwork board game.
    """

    # ================================ static attributes ================================

    AMOUNT_OF_ACTIONS = 2026
    """
    The amount of available actions for the game of patchwork. The actually allowed actions are way lower than this number, but we need to be able to represent all the possible actions in a single number. This is the maximum amount of actions that can be taken in a single turn.

    (MAX_PATCH_INDEX(2) * ROWS(9) + MAX_ROW(8)) * COLUMNS(9) + MAX_COLUMN(8)) * ROTATIONS(4) + MAX_ROTATION(3)) * ORIENTATIONS(2) + MAX_ORIENTATION(1) + ACTIONS_OTHER_THAN_NORMAL_PATCH_PLACEMENT_ACTION(83)
    """

    # ================================ instance attributes ================================

    id: int
    """The id of the action. This is a number between 0 and 2025 (both inclusive)."""

    patch: Optional[Patch]
    """The patch that is being placed."""

    patch_position: Optional[Position]
    """The position of the patch that is being placed."""

    patch_index: Optional[Literal[0, 1, 2]]
    """The index of the patch from the list of all available patches."""

    # ================================ instance properties ================================

    @property
    def is_walking(self) -> bool:
        """Whether this action is a walking action."""
        return self.id == 0

    @property
    def is_special_patch_placement(self) -> bool:
        """Whether this action is a special patch placement action."""
        return 1 <= self.id <= 81

    @property
    def is_patch_placement(self) -> bool:
        """Whether this action is a normal patch placement action."""
        return 82 <= self.id <= 2025

    @property
    def is_first_patch_taken(self) -> bool:
        """Whether this action took the first available patch."""
        return 82 <= self.id < 730

    @property
    def is_second_patch_taken(self) -> bool:
        """Whether this action took the second available patch."""
        return 730 <= self.id < 1378

    @property
    def is_third_patch_taken(self) -> bool:
        """Whether this action took the third available patch."""
        return 1378 <= self.id < 2026

    # ================================ static methods ================================

    @staticmethod
    def walking() -> Self:
        """Returns a walking action."""
        return Action(None, None, None)

    # ================================ instance methods ================================

    def __init__(self, patch: Optional[Patch], patch_position: Optional[Position], patch_index: Optional[Literal[0, 1, 2]]):
        self.patch = patch
        self.patch_position = patch_position
        self.patch_index = patch_index

        ROWS = 9
        COLUMNS = 9
        ROTATIONS = 4
        ORIENTATIONS = 2

        if patch is None:
            # walking action [0, 0]
            self.id = 0
        elif patch is not None and patch_position is not None and patch_index is None:
            OFFSET = 1
            # special patch placement action [1, 81]
            self.id = patch_position.row * COLUMNS + patch_position.column + OFFSET
        else:
            OFFSET = 82
            # the maximum amount of placement for a patch is actually 448. The patch is:
            # ▉
            # ▉▉▉
            # but as we want to be able to represent all the information in a single number, we need to use [(((index * ROWS + row) * COLUMNS + column) * ROTATIONS + rotation) * ORIENTATIONS + orientation + OFFSET] as limit for the action
            self.id = (
                patch_index * ROWS * COLUMNS * ROTATIONS * ORIENTATIONS +
                patch_position.row * COLUMNS * ROTATIONS * ORIENTATIONS +
                patch_position.column        * ROTATIONS * ORIENTATIONS +
                patch.rotation.value                     * ORIENTATIONS +
                patch.orientation.value +
                OFFSET
            )

    def __repr__(self) -> str:
        return f'Action(id={self.id}, patch={self.patch}, patch_position={self.patch_position}, patch_index={self.patch_index})'

    def __str__(self) -> str:
        action_str = f'Action {self.id}'

        if self.is_walking:
            action_str += ' - Walking'
            return action_str

        if self.is_special_patch_placement:
            action_str += f' - Special patch placement at ({self.patch_position.row}, {self.patch_position.column})'
            return action_str

        action_str += f' - Patch placement of patch at index {self.patch_index} at ({self.patch_position.row}, {self.patch_position.column})'

        return action_str

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Action) and self.id == other.id

    def __hash__(self) -> int:
        return hash(self.id)

    def __copy__(self) -> Self:
        return Action(self.patch, self.patch_position, self.patch_index)

    def __deepcopy__(self, memo: dict) -> Self:
        return Action(deepcopy(self.patch, memo), deepcopy(self.patch_position, memo), self.patch_index)

    def copy(self) -> Self:
        return deepcopy(self)


