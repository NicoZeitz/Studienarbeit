import { Link, LoaderFunctionArgs, useLoaderData } from 'react-router-dom';
import TableCenter from '../TableCenter/TableCenter.tsx';
import { API_URL, Game, type PatchworkState } from '../../constants.ts';
import { createContext } from 'react';
import Patch from '../Patch/Patch.tsx';
import { motion } from 'framer-motion';
import { ArrowClockwise, FlipHorizontal } from '@phosphor-icons/react';

export interface GameLayoutProps {}

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
                <div className="grow">
                    {/* TODO: absolute render patch queue here */}
                    {/* TODO: render patch selection here */}
                    <div className="fixed left-[375px] top-[30px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 0)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[240px] top-[0px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 1)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[100px] top-[-30px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 2)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[0px] top-[50px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 32)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[-20px] top-[180px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 4)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[0px] top-[320px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 5)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[-20px] top-[450px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 6)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[0px] top-[580px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 7)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[60px] top-[720px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 8)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[180px] top-[800px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 9)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[320px] top-[840px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 10)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[450px] top-[790px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 11)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[580px] top-[820px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 12)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[710px] top-[780px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 13)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[840px] top-[800px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 14)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[970px] top-[780px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 15)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1100px] top-[820px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 16)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1230px] top-[850px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 17)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1340px] top-[800px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 18)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1460px] top-[860px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 19)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    {/*Other side of the board (first 10 patches)*/}
                    <div className="fixed left-[1300px] top-[30px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 27)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1440px] top-[0px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 22)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1580px] top-[-30px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 23)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1700px] top-[50px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 24)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1680px] top-[180px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 25)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1700px] top-[320px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 26)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1680px] top-[450px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 21)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1700px] top-[580px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 28)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1660px] top-[710px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 29)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div className="fixed left-[1580px] top-[800px] scale-50">
                        <Patch
                            patch={state.patches.filter((p) => p.id === 31)[0]}
                            rotation={0}
                            flipped={false}
                        />
                    </div>
                    <div
                        style={{ gridTemplateRows: '1fr auto 1fr' }}
                        className="grid h-full grid-cols-1 items-center gap-0 pl-[15%] pr-[15%]"
                    >
                        <div className="gap-0ß relative ml-auto mr-auto flex">
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
                            {/* First patch to choose*/}
                            <div className="flex flex-col place-items-center">
                                <div className="flex">
                                    <FlipHorizontal
                                        size={32}
                                        weight="duotone"
                                    />
                                    <ArrowClockwise
                                        size={32}
                                        weight="duotone"
                                    />
                                </div>
                                <Patch
                                    patch={
                                        state.patches.filter(
                                            (p) => p.id === 20,
                                        )[0]
                                    }
                                    rotation={0}
                                    flipped={false}
                                />
                            </div>
                            {/* Second patch to choose*/}
                            <div className="flex flex-col place-items-center">
                                <div className="flex">
                                    <FlipHorizontal
                                        size={32}
                                        weight="duotone"
                                    />
                                    <ArrowClockwise
                                        size={32}
                                        weight="duotone"
                                    />
                                </div>
                                <Patch
                                    patch={
                                        state.patches.filter(
                                            (p) => p.id === 3,
                                        )[0]
                                    }
                                    rotation={0}
                                    flipped={false}
                                />
                            </div>
                            {/* Third patch to choose*/}
                            <div className="flex flex-col place-items-center">
                                <div className="flex">
                                    <FlipHorizontal
                                        size={32}
                                        weight="duotone"
                                    />
                                    <ArrowClockwise
                                        size={32}
                                        weight="duotone"
                                    />
                                </div>
                                <Patch
                                    patch={
                                        state.patches.filter(
                                            (p) => p.id === 30,
                                        )[0]
                                    }
                                    rotation={0}
                                    flipped={false}
                                />
                            </div>
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
                    className="fixed bottom-0 left-0 right-0 flex items-end justify-between"
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
