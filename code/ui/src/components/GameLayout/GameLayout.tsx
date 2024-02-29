import { Link, LoaderFunctionArgs, useLoaderData } from 'react-router-dom';
import TableCenter from '../TableCenter/TableCenter.tsx';
import { API_URL, Game, type PatchworkState } from '../../constants.ts';
import { createContext } from 'react';
import Patch from '../Patch/Patch.tsx';
import { motion } from 'framer-motion';

export interface GameLayoutProps { }

export const StateContext = createContext<PatchworkState>(null!);

export default function GameLayout(props: GameLayoutProps) {
    const game = useLoaderData() as Awaited<ReturnType<typeof gameLoader>>;
    const { state } = game;

    // TODO: extract player info to own component, define player colors somewhere
    return (
        <StateContext.Provider value={state}>
            <div
                style={
                    {
                        // backgroundImage: `url(/assets/background.png)`,
                        // backgroundRepeat: 'repeat-y',
                        // backgroundSize: 'contain',
                    }
                }
                className="relative flex h-dvh w-dvw flex-col overflow-clip"
            >
                <header>
                    HEADER PATCHWORK<Link to="/">NAVIGATE BACK</Link>
                </header>
                <div className="grow">
                    {/* TODO: absolute render patch queue here */}
                    {/* TODO: render patch selection here */}

                    <div
                        style={{ gridTemplateRows: '1fr auto 1fr' }}
                        className="grid h-full grid-cols-1 items-center gap-1 pl-[15%] pr-[15%]"
                    >
                        <div className="relative ml-auto mr-auto flex gap-2">
                            <div className="absolute left-[-10%] top-0 h-full w-[6%]">
                                <picture className="w-10% h-full object-contain drop-shadow-md">
                                    <source
                                        src="/assets/piece-marker.png"
                                        srcSet="/assets/piece-marker.png"
                                    />
                                    <img
                                        className="h-full w-full object-contain"
                                        src="/assets/piece-marker.png"
                                        alt={`Piece Marker`}
                                    />
                                </picture>
                            </div>
                            <Patch
                                patch={
                                    state.patches.filter((p) => p.id === 10)[0]
                                }
                                rotation={0}
                                flipped={false}
                            />
                            <Patch
                                patch={
                                    state.patches.filter((p) => p.id === 10)[0]
                                }
                                rotation={0}
                                flipped={false}
                            />
                            <Patch
                                patch={
                                    state.patches.filter((p) => p.id === 10)[0]
                                }
                                rotation={0}
                                flipped={false}
                            />
                        </div>
                        <div className="w-full ">
                            <TableCenter />
                        </div>
                    </div>
                </div>
                <motion.footer
                    initial={{ translateY: 100 }}
                    animate={{ translateY: 0 }}
                    transition={{ duration: 0.75 }}
                    className="flex justify-between fixed left-0 right-0 bottom-0 items-end"
                >
                    <motion.div
                        initial={{ translateX: -100 }}
                        animate={{ translateX: 0 }}
                        transition={{ duration: 0.75 }}
                        className="h-full rounded-tr-2xl bg-[#b2bba3] pb-2 pl-2 pr-5 pt-5"
                    >
                        PLAYER 1 INFORMATION
                    </motion.div>
                    <div className="mb-4 flex flex-col items-center gap-1">
                        <span className="text-2xl">
                            Zug von Spieler 1 (grün)
                        </span>
                        <span className="from-neutral-500">Zug 42</span>
                    </div>
                    <motion.div
                        initial={{ translateX: 100 }}
                        animate={{ translateX: 0 }}
                        transition={{ duration: 0.75 }}
                        className="h-full rounded-tl-2xl bg-[#cac792] pb-2 pl-5 pr-2 pt-5"
                    >
                        PLAYER 2 INFORMATION
                    </motion.div>
                </motion.footer>
            </div>
        </StateContext.Provider>
    );
}

// eslint-disable-next-line react-refresh/only-export-components
export async function gameLoader({ params }: LoaderFunctionArgs) {
    const { id } = params as { id: string };

    // TODO: other url to start or continue a game
    // const res = await fetch(`${API_URL}/game/${id}`);
    const res = await fetch(`${API_URL}/game/${id}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        // body: JSON.stringify({ parameters }),
    });
    const game = (await res.json()) as Game;

    console.log(id);
    console.log(game); // TODO: remove

    return game;
}
