import cProfile
from patchwork import GameLoop, RandomPlayer, MCTSPlayer

if __name__ == '__main__':
    game_loop = GameLoop()

    with cProfile.Profile() as p:
        game_loop.test(
            player_1=MCTSPlayer(name="Player 1 (MCTS)"),
            # player_1=RandomPlayer(name="Player 1 (Random)"),
            player_2=RandomPlayer(name="Player 2 (Random)")
        )
        p.print_stats(sort="cumtime")
