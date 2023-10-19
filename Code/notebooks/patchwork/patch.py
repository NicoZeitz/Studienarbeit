from dataclasses import dataclass
from typing import List, Union, Self
import random

import numpy as np

# TODO: save rotations for backside + save ids
@dataclass
class Patch:
    id: int
    tiles: np.ndarray
    button_cost: int
    time_cost: int
    button_income: int

    @staticmethod
    def get_special_piece() -> Self:
        return Patch(
            0, # TODO: each special piece should have a unique id
            np.array([
                [1]
            ]),
            button_cost=0,
            time_cost=0,
            button_income=0
        )

    @staticmethod
    def generate_pieces(seed: Union[int, None] = None) -> List[Self]:
        pieces = [
            Patch(
                1,
                np.array([
                    [1,0,0],
                    [1,1,0],
                    [0,1,1],
                ]),
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
                ]),
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
                ]),
                button_cost=8,
                time_cost=6,
                button_income=3
            ),
            Patch(
                4,
                np.array([
                    [0,1,1],
                    [1,1,0]
                ]),
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
                ]),
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
                ]),
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
                ]),
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
                ]),
                button_cost=2,
                time_cost=2,
                button_income=0
            ),
            Patch(
                9,
                np.array([
                    [1,1],
                    [1,1]
                ]),
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
                ]),
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
                ]),
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
                ]),
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
                ]),
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
                ]),
                button_cost=4,
                time_cost=6,
                button_income=2
            ),
            Patch(
                15,
                np.array([
                    [0,1,1,0],
                    [1,1,1,1]
                ]),
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
                ]),
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
                ]),
                button_cost=5,
                time_cost=4,
                button_income=2
            ),
            Patch(
                18,
                np.array([
                    [1,0,0,0],
                    [1,1,1,1]
                ]),
                button_cost=10,
                time_cost=3,
                button_income=2
            ),
            Patch(
                19,
                np.array([
                    [0,0,1],
                    [1,1,1]
                ]),
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
                ]),
                button_cost=1,
                time_cost=4,
                button_income=1
            ),
            Patch(
                21,
                np.array([
                    [0,1],
                    [1,1]
                ]),
                button_cost=1,
                time_cost=3,
                button_income=0
            ),
            Patch(
                22,
                np.array([
                    [1,0,1],
                    [1,1,1]
                ]),
                button_cost=1,
                time_cost=2,
                button_income=0
            ),
            Patch(
                23,
                np.array([
                    [0,1],
                    [1,1]
                ]),
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
                ]),
                button_cost=2,
                time_cost=2,
                button_income=0
            ),
            Patch(
                25,
                np.array([
                    [1,1,1]
                ]),
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
                ]),
                button_cost=3,
                time_cost=2,
                button_income=1
            ),
            Patch(
                27,
                np.array([
                    [1,1,1,1,1]
                ]),
                button_cost=7,
                time_cost=1,
                button_income=1
            ),
            Patch(
                28,
                np.array([
                    [1,1,1,1]
                ]),
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
                ]),
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
                ]),
                button_cost=3,
                time_cost=6,
                button_income=2
            ),
            Patch(
                31,
                np.array([
                    [0,0,1,0],
                    [1,1,1,1]
                ]),
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
                ]),
                button_cost=0,
                time_cost=3,
                button_income=1
            )
        ]
        if seed is not None:
            random.Random(seed).shuffle(pieces)
        else:
            random.shuffle(pieces)
        # add starting piece to end
        pieces.append(Patch(
            33,
            np.array([
                [1, 1]
            ]),
            button_cost=2,
            time_cost=1,
            button_income=0
        ))
        return pieces

    def get_unique_symmetries(self) -> List[np.array]:
        patch_tiles = self.tiles
        patch_flipped_tiles = np.flip(patch_tiles, axis=0)

        patch_symmetries = [
            patch_tiles,
            np.rot90(patch_tiles),
            np.rot90(patch_tiles, k=2),
            np.rot90(patch_tiles, k=3),
            patch_flipped_tiles,
            np.rot90(patch_flipped_tiles),
            np.rot90(patch_flipped_tiles, k=2),
            np.rot90(patch_flipped_tiles, k=3)
        ]

        unique_patch_symmetries = []
        for symmetry in patch_symmetries:
            if next((False for unique_symmetry in unique_patch_symmetries if np.array_equal(unique_symmetry, symmetry)), True):
                unique_patch_symmetries.append(symmetry)

        return unique_patch_symmetries

    def __str__(self) -> str:
        tile_str = ""
        for row in range(np.size(self.tiles, 0)):
            for column in range(np.size(self.tiles, 1)):
                tile_str += "█" if self.tiles[row, column] == 1 else " "

            if row == np.size(self.tiles, 0) - 2 or np.size(self.tiles, 0) == 1:
                tile_str += f" income: {self.button_income}, button_cost {self.button_cost}, time_cost: {self.time_cost}"
            tile_str += "\n"

        return tile_str

if __name__ == "__main__":
    for piece in Patch.generate_pieces():
        print(piece)