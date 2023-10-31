from dataclasses import dataclass
from enum import Enum
from typing import List, Self, Optional, Union
import random

import numpy as np
import numpy.typing as npt

class Rotation(Enum):
    ZERO = 0
    NINETY = 1
    ONE_EIGHTY = 2
    TWO_SEVENTY = 3

class Orientation(Enum):
    NORMAL = 0
    FLIPPED = 1

class PatchTransformation:
    _data: int

    @property
    def rotation(self) -> Rotation:
        return Rotation(self._data & 0b11)

    @property
    def orientation(self) -> Orientation:
        return Orientation(self._data & 0b100 != 0)

    def __init__(self, rotation: Rotation, orientation: Orientation):
        self._data = rotation.value | (orientation.value << 2)

    def __eq__(self, other: Self) -> bool:
        return self._data == other._data

    def __hash__(self) -> int:
        return hash(self._data)

    def __repr__(self) -> str:
        return f"PatchTransformation(rotation={self.rotation}, orientation={self.orientation})"

    def __str__(self) -> str:
        return f'({self.rotation.value * 90}° {self.orientation.name})'

    def __copy__(self) -> Self:
        return PatchTransformation(self.rotation, self.orientation)

    def copy(self) -> Self:
        return self.__copy__()

class PatchImage:
    pass

@dataclass
class Patch:
    """
    Represents a patch in the game Patchwork.
    """

    # ================================ instance attributes ================================
    id: int
    """The unique id of the patch"""

    tiles: Union[
        # 5xn
        np.ndarray[(5,3), bool],
        np.ndarray[(5,1), bool],
        # 4xn
        np.ndarray[(4,4), bool],
        np.ndarray[(4,3), bool],
        np.ndarray[(4,2), bool],
        np.ndarray[(4,1), bool],
        # 3xn
        np.ndarray[(3,5), bool],
        np.ndarray[(3,4), bool],
        np.ndarray[(3,3), bool],
        np.ndarray[(3,2), bool],
        np.ndarray[(3,1), bool],
        # 2xn
        np.ndarray[(2,4), bool],
        np.ndarray[(2,3), bool],
        np.ndarray[(2,2), bool],
        np.ndarray[(2,1), bool],
        # 1xn
        np.ndarray[(1,5), bool],
        np.ndarray[(1,4), bool],
        np.ndarray[(1,3), bool],
        np.ndarray[(1,2), bool],
        np.ndarray[(1,1), bool],
    ]
    """The tiles of the patch"""

    button_cost: int
    """The amount of buttons it costs to buy this patch"""

    time_cost: int
    """The amount of time it costs to buy this patch"""

    button_income: int
    """The amount of buttons you get as additional income from this patch"""

    transformation: PatchTransformation = PatchTransformation(Rotation.ZERO, Orientation.NORMAL)
    """The transformation of the patch. Different transformations of the same patch are considered the same patch"""

    # ================================ instance properties ================================

    @property
    def is_special(self) -> bool:
        """Returns whether this patch is a special patch"""
        return self.id >= Patch.amount_of_patches()

    @property
    def is_starting_patch(self) -> bool:
        """Returns whether this patch is the starting patch"""
        return self.id == 0

    @property
    def shape(self) -> tuple[int, int]:
        """Returns the shape of the patch"""
        return np.shape(self.tiles)

    @property
    def rotation(self) -> Rotation:
        """Returns the rotation of the patch"""
        return self.transformation.rotation

    @property
    def orientation(self) -> Orientation:
        """Returns the orientation of the patch"""
        return self.transformation.orientation

    @property
    def front_image(self) -> PatchImage:
        # TODO: return the front image with correct rotation and orientation information
        pass

    @property
    def back_image(self) -> PatchImage:
        # TODO: return the back image with correct rotation and orientation information
        pass

    # ================================ private instance attributes ================================

    _unique_transformations: Optional[List[Self]] = None

    # ================================ private static attributes ================================

    _len_patches: Optional[int] = None

    # ================================ static methods ================================

    @staticmethod
    def get_special_patch(index: int) -> Self:
        # TODO: index is still wrong and needs to be converted
        return Patch(
            Patch.amount_of_patches() + index,
            np.array([
                [1]
            ], dtype=bool),
            button_cost=0,
            time_cost=0,
            button_income=0
        )

    @staticmethod
    def amount_of_patches() -> int:
        """
        Returns the amount of patches in the game (excluding special patches)

        :return: The amount of patches in the game (excluding special patches)
        """
        if Patch._len_patches is None:
            Patch._len_patches = len(Patch._patches()) + 1

        return Patch._len_patches

    @staticmethod
    def generate_patches(seed: Optional[int] = None) -> List[Self]:
        """
        Generates all patches in the game (excluding special patches) and shuffles them randomly.

        :param seed: The seed to use for the random shuffle. If None, no seed is used.
        :return: A list of all patches in the game (excluding special patches) in a random order.
        """
        patches = Patch._patches()
        random.Random(seed).shuffle(patches)
        patches.append(Patch._starting_patch())
        return patches

    # ================================ instance methods ================================

    def __init__(
            self,
            id: int,
            tiles: npt.NDArray[np.bool_],
            button_cost: int,
            time_cost: int,
            button_income: int,
            transformation: PatchTransformation = PatchTransformation(Rotation.ZERO, Orientation.NORMAL)
        ):
        self.id = id
        self.tiles = tiles
        self.button_cost = button_cost
        self.time_cost = time_cost
        self.button_income = button_income
        self.transformation = transformation
        if transformation.rotation == Rotation.ZERO and transformation.orientation == Orientation.NORMAL:
            self.get_unique_transformations()

    def get_unique_transformations(self) -> List[Self]:
        """
        Returns all unique transformations (rotations and reflections) of this patch.
        """
        if self._unique_transformations is not None:
            return self._unique_transformations

        unique_transformations = [
            self
        ]

        transformed_tiles = self.tiles
        for orientation in Orientation:
            for rotation in Rotation:
                if rotation == Rotation.ZERO and orientation == Orientation.NORMAL:
                    continue

                transformed_tiles = self.tiles
                if orientation == Orientation.FLIPPED:
                    transformed_tiles = np.flip(transformed_tiles, axis=0)
                transformed_tiles = np.rot90(transformed_tiles, k=rotation.value)

                duplicate_transformation = False
                for transformation in unique_transformations:
                    if np.array_equal(transformation.tiles, transformed_tiles):
                        duplicate_transformation = True

                if not duplicate_transformation:
                    unique_transformations.append(Patch(
                        self.id,
                        transformed_tiles,
                        self.button_cost,
                        self.time_cost,
                        self.button_income,
                        PatchTransformation(rotation, orientation)
                    ))

        for p in unique_transformations:
            p._unique_transformations = unique_transformations

        return self._unique_transformations

    def __eq__(self, other: Self) -> bool:
        return self.id == other.id

    def __hash__(self) -> int:
        return hash(self.id)

    def __repr__(self) -> str:
        return f"Patch(id={self.id}, button_cost={self.button_cost}, time_cost={self.time_cost}, button_income={self.button_income}, transformation={self.transformation}, tiles={self.tiles})"

    def __str__(self) -> str:
        patch_str = ""
        for row in range(np.size(self.tiles, 0)):
            for column in range(np.size(self.tiles, 1)):
                patch_str += "█" if self.tiles[row, column] == 1 else " "

            patch_str += "\n"

        patch_str += f'Income: {self.button_income}\n'
        patch_str += f'Button cost: {self.button_cost}\n'
        patch_str += f'Time cost: {self.time_cost}\n'

        return patch_str

    def __copy__(self) -> Self:
        return Patch(
            self.id,
            self.tiles,
            self.button_cost,
            self.time_cost,
            self.button_income,
            self.transformation
        )

    def __deepcopy__(self, memo) -> Self:
        return Patch(
            self.id,
            np.copy(self.tiles),
            self.button_cost,
            self.time_cost,
            self.button_income,
            self.transformation
        )

    def copy(self) -> Self:
        return self.__copy__()

    # ================================ private static methods ================================

    @staticmethod
    def _starting_patch() -> Self:
        return Patch(
            0,
            np.array([
                [1, 1]
            ], dtype=bool),
            button_cost=2,
            time_cost=1,
            button_income=0
        )

    @staticmethod
    def _patches() -> List[Self]:
        return [
            Patch(1, np.array([
                    [1,0,0],
                    [1,1,0],
                    [0,1,1],
                ], dtype=bool),
                button_cost=10,
                time_cost=4,
                button_income=3
                ),
            Patch(
                2,
                np.array([
                    [0,1,1,1,0],
                    [1,1,1,1,1],
                    [0,1,1,1,0]
                ], dtype=bool),
                button_cost=5,
                time_cost=3,
                button_income=1
            ),
            Patch(
                3,
                np.array([
                    [0,1,1],
                    [0,1,1],
                    [1,1,0]
                ], dtype=bool),
                button_cost=8,
                time_cost=6,
                button_income=3
            ),
            Patch(
                4,
                np.array([
                    [0,1,1],
                    [1,1,0]
                ], dtype=bool),
                button_cost=7,
                time_cost=6,
                button_income=3
            ),
            Patch(
                5,
                np.array([
                    [1,0],
                    [1,1],
                    [1,1],
                    [0,1]
                ], dtype=bool),
                button_cost=4,
                time_cost=2,
                button_income=0
            ),
            Patch(
                6,
                np.array([
                    [0,1,0],
                    [0,1,1],
                    [1,1,0],
                    [0,1,0]
                ], dtype=bool),
                button_cost=2,
                time_cost=1,
                button_income=0
            ),
            Patch(
                7,
                np.array([
                    [1,0,1],
                    [1,1,1],
                    [1,0,1]
                ], dtype=bool),
                button_cost=2,
                time_cost=3,
                button_income=0
            ),
            Patch(
                8,
                np.array([
                    [1,0],
                    [1,1],
                    [1,1]
                ], dtype=bool),
                button_cost=2,
                time_cost=2,
                button_income=0
            ),
            Patch(
                9,
                np.array([
                    [1,1],
                    [1,1]
                ], dtype=bool),
                button_cost=6,
                time_cost=5,
                button_income=2
            ),
            Patch(
                10,
                np.array([
                    [0,1],
                    [0,1],
                    [1,1],
                    [1,0]
                ], dtype=bool),
                button_cost=2,
                time_cost=3,
                button_income=1
            ),
            Patch(
                11,
                np.array([
                    [0,0,0,1],
                    [1,1,1,1],
                    [1,0,0,0]
                ], dtype=bool),
                button_cost=1,
                time_cost=2,
                button_income=0
            ),
            Patch(
                12,
                np.array([
                    [1,1],
                    [1,1],
                    [0,1],
                    [0,1],
                ], dtype=bool),
                button_cost=10,
                time_cost=5,
                button_income=3
            ),
            Patch(
                13,
                np.array([
                    [0,1,0],
                    [0,1,0],
                    [0,1,0],
                    [1,1,1]
                ], dtype=bool),
                button_cost=7,
                time_cost=2,
                button_income=2
            ),
            Patch(
                14,
                np.array([
                    [0,1],
                    [0,1],
                    [1,1]
                ], dtype=bool),
                button_cost=4,
                time_cost=6,
                button_income=2
            ),
            Patch(
                15,
                np.array([
                    [0,1,1,0],
                    [1,1,1,1]
                ], dtype=bool),
                button_cost=7,
                time_cost=4,
                button_income=2
            ),
            Patch(
                16,
                np.array([
                    [1,1],
                    [0,1],
                    [0,1],
                    [1,1]
                ], dtype=bool),
                button_cost=1,
                time_cost=5,
                button_income=1
            ),
            Patch(
                17,
                np.array([
                    [0,1,0],
                    [1,1,1],
                    [0,1,0]
                ], dtype=bool),
                button_cost=5,
                time_cost=4,
                button_income=2
            ),
            Patch(
                18,
                np.array([
                    [1,0,0,0],
                    [1,1,1,1]
                ], dtype=bool),
                button_cost=10,
                time_cost=3,
                button_income=2
            ),
            Patch(
                19,
                np.array([
                    [0,0,1],
                    [1,1,1]
                ], dtype=bool),
                button_cost=4,
                time_cost=2,
                button_income=1
            ),
            Patch(
                20,
                np.array([
                    [0,0,1,0,0],
                    [1,1,1,1,1],
                    [0,0,1,0,0]
                ], dtype=bool),
                button_cost=1,
                time_cost=4,
                button_income=1
            ),
            Patch(
                21,
                np.array([
                    [0,1],
                    [1,1]
                ], dtype=bool),
                button_cost=1,
                time_cost=3,
                button_income=0
            ),
            Patch(
                22,
                np.array([
                    [1,0,1],
                    [1,1,1]
                ], dtype=bool),
                button_cost=1,
                time_cost=2,
                button_income=0
            ),
            Patch(
                23,
                np.array([
                    [0,1],
                    [1,1]
                ], dtype=bool),
                button_cost=3,
                time_cost=1,
                button_income=0
            ),
            Patch(
                24,
                np.array([
                    [0,1],
                    [1,1],
                    [0,1]
                ], dtype=bool),
                button_cost=2,
                time_cost=2,
                button_income=0
            ),
            Patch(
                25,
                np.array([
                    [1,1,1]
                ], dtype=bool),
                button_cost=2,
                time_cost=2,
                button_income=0
            ),
            Patch(
                26,
                np.array([
                    [0,1],
                    [1,1],
                    [1,0]
                ], dtype=bool),
                button_cost=3,
                time_cost=2,
                button_income=1
            ),
            Patch(
                27,
                np.array([
                    [1,1,1,1,1]
                ], dtype=bool),
                button_cost=7,
                time_cost=1,
                button_income=1
            ),
            Patch(
                28,
                np.array([
                    [1,1,1,1]
                ], dtype=bool),
                button_cost=3,
                time_cost=3,
                button_income=1
            ),
            Patch(
                29,
                np.array([
                    [0,1,0],
                    [0,1,0],
                    [1,1,1]
                ], dtype=bool),
                button_cost=5,
                time_cost=5,
                button_income=2
            ),
            Patch(
                30,
                np.array([
                    [0,1,0],
                    [1,1,1],
                    [1,0,1]
                ], dtype=bool),
                button_cost=3,
                time_cost=6,
                button_income=2
            ),
            Patch(
                31,
                np.array([
                    [0,0,1,0],
                    [1,1,1,1]
                ], dtype=bool),
                button_cost=3,
                time_cost=4,
                button_income=1
            ),
            Patch(
                32,
                np.array([
                    [0,1,0,0],
                    [1,1,1,1],
                    [0,1,0,0],
                ], dtype=bool),
                button_cost=0,
                time_cost=3,
                button_income=1
            )
        ]