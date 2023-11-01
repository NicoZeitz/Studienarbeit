import cProfile
from patchwork import GameLoop, RandomPlayer, MCTSPlayer

game_loop = GameLoop()

with cProfile.Profile() as p:
    game_loop.run(
        player_1=MCTSPlayer(name="Player 1 (MCTS)"),
        player_2=RandomPlayer(name="Player 2 (Random)")
    )
    p.print_stats(sort="cumtime")
