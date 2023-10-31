from typing import TypedDict

class MCTSOptions(TypedDict):
    C: float
    """The exploration parameter for the UCT (Upper Confidence Bound 1 applied to trees) algorithm."""

    number_of_simulations: int
    """The number of simulations to run."""
