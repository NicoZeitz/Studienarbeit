export interface TimePieceProps {
    player: 1 | 2;
}

export default function TimePiece(props: TimePieceProps) {
    const { player } = props;
    const src = `/assets/figure-player-${player}.png`;

    return (
        <div className="grid h-full w-full place-items-center">
            {/* TODO: better shadow */}
            <picture
                style={
                    {
                        '--tw-drop-shadow':
                            'drop-shadow(0 4px 3px rgb(0 0 0 / 0.17)) drop-shadow(0 2px 2px rgb(0 0 0 / 0.16))',
                    } as React.CSSProperties
                }
                className="h-[60%] w-[60%] overflow-clip rounded-full drop-shadow-md"
            >
                <source src={src} srcSet={src} />
                <img src={src} alt={`Player ${player} Time Piece`} />
            </picture>
        </div>
    );
}
