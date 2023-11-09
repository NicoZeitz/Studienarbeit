import cProfile
from patchwork import GameLoop, RandomPlayer, MCTSPlayer, MinimaxPlayer

if __name__ == '__main__':
    game_loop = GameLoop()

    with cProfile.Profile() as p:
        game_loop.run(
            player_1=MinimaxPlayer(name="Player 1 (Minimax)"),
            # player_1=RandomPlayer(name="Player 1 (Random)"),
            player_2=RandomPlayer(name="Player 2 (Random)")
        )
        p.print_stats(sort="cumtime")
