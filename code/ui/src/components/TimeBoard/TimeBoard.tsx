import TimePiece from './TimePiece.tsx';
import { timeBoardMapping } from './timeBoardMapping.ts';

export interface TimeBoardProps {
    player1Position: number;
    player2Position: number;
    player: 1 | 2;
    // currentPlayer: 'player1' | 'player2';
}

// TODO: special patches
// TODO: coordinates of button income trigger

function getTranslate(position: number, player: 1 | 2): string {
    return '';
}

// TODO: drag and drop and accept function which gives the position
export default function TimeBoard(props: TimeBoardProps) {
    const { player, player1Position, player2Position } = props;

    const currentPlayerPosition =
        player === 1 ? player1Position : player2Position;
    const otherPlayerPosition =
        player === 1 ? player2Position : player1Position;

    const currentPlayerTransform = `translate(${timeBoardMapping[currentPlayerPosition].left}, ${timeBoardMapping[currentPlayerPosition].top})`;
    const otherPlayerTransform = `translate(${timeBoardMapping[otherPlayerPosition].left}, ${timeBoardMapping[otherPlayerPosition].top})`;
    // TODO: if player 2 at pos 0 than transform + 10% top

    return (
        <div className="relative aspect-square w-full">
            <picture className="h-full w-full drop-shadow-lg">
                <source
                    src="/assets/time-board.jpg"
                    srcSet="/assets/time-board.jpg"
                />
                <img src="/assets/time-board.jpg" alt="Time Board" />
            </picture>
            <div
                style={{ transform: otherPlayerTransform }}
                className="absolute left-0 top-0 h-[12.5%] w-[12.5%] transition-transform duration-200"
            >
                <TimePiece player={((player % 2) + 1) as 1 | 2} />
            </div>
            {/* Current player is later to be rendered on top */}
            <div
                style={{ transform: currentPlayerTransform }}
                className="absolute left-[3px] top-[3px] h-[12.5%] w-[12.5%] transition-transform duration-200"
            >
                <TimePiece player={player} />
            </div>
        </div>
    );
}
