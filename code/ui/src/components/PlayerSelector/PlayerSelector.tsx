import { CaretDown, GearSix } from '@phosphor-icons/react';
import * as Popover from '@radix-ui/react-popover';
import * as Select from '@radix-ui/react-select';
import * as Separator from '@radix-ui/react-separator';
import PopupSettings from '../PopupSettings/PopupSettings';
import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import {
    PlayerSettings,
    playerSettings,
} from '../PopupSettings/playerSettings.ts';
import { PlayerIds } from '../../player/playerIds.ts';

export interface PlayerSelectorProps {
    label: string;
    playerNumber: 1 | 2;
    player: {
        selectedPlayer: null | string;
        settings: PlayerSettings;
    };
}

export type SettingsState = {
    [id in keyof typeof playerSettings]: {
        [id: string]: string | number | boolean | undefined;
    };
};

export default function PlayerSelector(props: PlayerSelectorProps) {
    const [player, setPlayer] = useState<PlayerIds | null>(null);
    const [settingsState, setSettingsState] = useState<SettingsState>({
        Mensch: {},
        'KI-Random': {},
        'KI-Greedy': {},
        'KI-Minimax': {},
        'KI-PVS': {},
        'KI-MCTS': {},
        'KI-AlphaZero': {},
    });

    const selectId = `player-${props.playerNumber}-select`;

    return (
        <div className="flex flex-col">
            <label className="text-lg" htmlFor={selectId}>
                {props.label}
            </label>
            <div className="flex gap-2 rounded-lg p-3 shadow-lg">
                <Select.Root
                    required
                    onValueChange={(player) => setPlayer(player as PlayerIds)}
                >
                    <Select.Trigger
                        className="flex w-[29ch] items-center justify-between text-lg data-[placeholder]:text-red-500"
                        id={selectId}
                    >
                        <Select.Value
                            placeholder={`Spieler ${props.playerNumber} wählen`}
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
                    <Popover.Trigger disabled={player === null}>
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
                                    <PopupSettings
                                        playerType={player!}
                                        player={props.playerNumber}
                                        settingsState={settingsState}
                                        setSettingsState={setSettingsState}
                                    />
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
