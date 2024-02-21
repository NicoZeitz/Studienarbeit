import { CaretDown, GearSix } from '@phosphor-icons/react';
import * as Select from '@radix-ui/react-select';
import * as Separator from '@radix-ui/react-separator';

export interface PlayerSelectorProps {
    label: string;
    player: 1 | 2;
}

export default function PlayerSelector(props: PlayerSelectorProps) {
    const selectId = `player-${props.player}-select`;

    return (
        <div className="flex flex-col">
            <label className="text-lg" htmlFor={selectId}>
                {props.label}
            </label>
            <div className="flex gap-2 rounded-lg p-3 shadow-lg">
                <Select.Root>
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
                <GearSix size={32} weight="duotone" />
            </div>
        </div>
    );
}
