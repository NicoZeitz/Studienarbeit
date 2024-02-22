import { CaretDown, GearSix } from '@phosphor-icons/react';
import * as Popover from '@radix-ui/react-popover';
import * as Select from '@radix-ui/react-select';
import * as Separator from '@radix-ui/react-separator';
import PopupSettings from '../PopupSettings/PopupSettings';
import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';

export interface PlayerSelectorProps {
    label: string;
    player: 1 | 2;
}

type PlayerIds =
    | 'Mensch'
    | 'KI-Random'
    | 'KI-Greedy'
    | 'KI-Minimax'
    | 'KI-PVS'
    | 'KI-MCTS'
    | 'KI-AlphaZero';

export default function PlayerSelector(props: PlayerSelectorProps) {
    const selectId = `player-${props.player}-select`;
    const [player, setPlayer] = useState<PlayerIds>('Mensch');

    return (
        <div className="flex flex-col">
            <label className="text-lg" htmlFor={selectId}>
                {props.label}
            </label>
            <div className="flex gap-2 rounded-lg p-3 shadow-lg">
                <Select.Root
                    onValueChange={(player) => setPlayer(player as PlayerIds)}
                >
                    <Select.Trigger
                        className="flex w-[29ch] items-center justify-between text-lg data-[placeholder]:text-red-500"
                        id={selectId}
                    >
                        <Select.Value
                            placeholder={`Spieler ${props.player} wählen`}
                        />
                        <Select.Icon className="text-black">
                            <CaretDown size={32} weight="duotone" />
                        </Select.Icon>
                    </Select.Trigger>

                    <Select.Portal>
                        <Select.Content className="rounded-md bg-white p-5 shadow-xl">
                            <Select.Viewport className="flex flex-col gap-2 text-lg">
                                <Select.Item value="Mensch">
                                    <Select.ItemText>Mensch</Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-Random">
                                    <Select.ItemText>
                                        KI: Zufallszüge
                                    </Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-Greedy">
                                    <Select.ItemText>
                                        KI: Greedy
                                    </Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-Minimax">
                                    <Select.ItemText>
                                        KI: Minimax
                                    </Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-PVS">
                                    <Select.ItemText>
                                        KI: Principle Variation Search
                                    </Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-MCTS">
                                    <Select.ItemText>
                                        KI: Monte Carlo Tree Search
                                    </Select.ItemText>
                                </Select.Item>
                                <Select.Item value="KI-AlphaZero">
                                    <Select.ItemText>
                                        KI: AlphaZero
                                    </Select.ItemText>
                                </Select.Item>
                            </Select.Viewport>
                        </Select.Content>
                    </Select.Portal>
                </Select.Root>
                <Separator.Root
                    className="h-full w-px bg-black"
                    decorative
                    orientation="vertical"
                />

                <Popover.Root>
                    <Popover.Trigger asChild>
                        <GearSix size={32} weight="duotone" />
                    </Popover.Trigger>
                    <AnimatePresence>
                        <Popover.Portal>
                            <Popover.Content
                                asChild={true}
                                side="top"
                                className="rounded-lg bg-white p-5 shadow-xl"
                                sideOffset={17}
                            >
                                <motion.div
                                    initial={{ opacity: 0 }}
                                    animate={{ opacity: 1 }}
                                    transition={{ duration: 0.15 }}
                                    exit={{ opacity: 0 }}
                                >
                                    <PopupSettings selectedPlayer={player} />
                                    <Popover.Arrow className="fill-white" />
                                </motion.div>
                            </Popover.Content>
                        </Popover.Portal>
                    </AnimatePresence>
                </Popover.Root>
            </div>
        </div>
    );
}
