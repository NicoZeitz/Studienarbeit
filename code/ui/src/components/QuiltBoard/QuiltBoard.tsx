export interface QuiltBoardProps {
    player: 1 | 2;
}

export default function QuiltBoard(props: QuiltBoardProps) {
    const { player } = props;
    const src = `/assets/board-player-${player}.jpg`;

    return (
        <div className="grid aspect-square w-full place-items-center">
            <picture className="h-full w-full overflow-clip drop-shadow-lg">
                <source src={src} srcSet={src} />
                <img src={src} alt={`Player ${player} Quilt Board`} />
            </picture>
        </div>
    );
}
