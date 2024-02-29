import * as Slider from '@radix-ui/react-slider';
import * as Switch from '@radix-ui/react-switch';
import * as Select from '@radix-ui/react-select';
import React from 'react';
import { useState } from 'react';
import { CaretDown } from '@phosphor-icons/react';

export interface PopupSettingsProps {
    selectedPlayer: keyof typeof settings;
}

const settings = {
    Mensch: [{ name: 'Name', type: 'Text-Input' }],
    'KI-Random': [
        { name: 'Name', type: 'Text-Input' },
        {
            name: 'Seed',
            type: 'Number-Input',
            min: 0,
            max: 1000,
            defaultValue: (Math.random() * 1000 + 1) | 0,
        },
    ],
    'KI-Greedy': [
        { name: 'Name', type: 'Text-Input' },
        {
            name: 'Evaluierer',
            type: 'Select',
            options: [
                { value: 'Statischer Evaluierer', id: 'static' },
                { value: 'Zufallsspiel', id: 'win' },
                { value: 'Bewertetes Zufallsspiel', id: 'score' },
                { value: 'Neuronales Netz', id: 'nn' },
            ],
        },
    ],
    'KI-Minimax': [
        { name: 'Name', type: 'Text-Input' },
        { name: 'Tiefe', type: 'Number-Input', min: 1, max: 20 },
        { name: 'Flickenzüge', type: 'Number-Input', min: 1, max: 10 },
    ],
    'KI-PVS': [
        { name: 'Name', type: 'Text-Input' },
        { name: 'Zugzeit in Sekunden', type: 'Number-Input', min: 1, max: 60 },
        {
            name: 'Evaluierer',
            type: 'Select',
            options: [
                { value: 'Statischer Evaluierer', id: 'static' },
                { value: 'Zufallsspiel', id: 'win' },
                { value: 'Bewertetes Zufallsspiel', id: 'score' },
                { value: 'Neuronales Netz', id: 'nn' },
            ],
        },
        {
            name: 'Failing strategy',
            type: 'Select',
            options: [
                { value: 'Soft', id: 'soft' },
                { value: 'Hard', id: 'hard' },
            ],
        },
        { name: 'Aspiration window', type: 'Checkbox', defaultValue: false },
        { name: 'Late Move Reduction', type: 'Checkbox', defaultValue: true },
        { name: 'Late Move Pruning', type: 'Checkbox', defaultValue: false },
        { name: 'Transposition Table', type: 'Checkbox', defaultValue: false },
        { name: 'Lazy-SMP', type: 'Checkbox', defaultValue: false },
    ],
    'KI-MCTS': [
        { name: 'Name', type: 'Text-Input' },
        { name: 'Zugzeit in Sekunden', type: 'Number-Input', min: 1, max: 60 },
        { name: 'Baum wiederverwenden', type: 'Checkbox', defaultValue: false },
        {
            name: 'Wurzeln parallelisieren',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            name: 'Blätter parallelisieren',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            name: 'Tree Policy',
            type: 'Select',
            options: [
                { value: 'UCT', id: 'uct' },
                { value: 'Partial Score', id: 'partial-score' },
                { value: 'Score', id: 'score' },
            ],
        },
        {
            name: 'Evaluierer',
            type: 'Select',
            options: [
                { value: 'Statischer Evaluierer', id: 'static' },
                { value: 'Zufallsspiel', id: 'win' },
                { value: 'Bewertetes Zufallsspiel', id: 'score' },
                { value: 'Neuronales Netz', id: 'nn' },
            ],
        },
    ],
    'KI-AlphaZero': [{ name: 'Name', type: 'Text-Input' }],
} as const satisfies { name: string; type: string; defaultValue?: string };

export default function PopupSettings(props: PopupSettingsProps) {
    const currentSettings = settings[props.selectedPlayer];

    return (
        <div>
            <h1 className="mb-2 font-bold">Einstellungen</h1>
            <div className="grid grid-cols-[minmax(0,1fr)_30ch] place-items-end gap-x-4 gap-y-2">
                {currentSettings.map((setting, index) => {
                    let id = 'test';
                    return (
                        <React.Fragment key={index}>
                            <label
                                htmlFor={id}
                                className="place-self-start"
                                style={{ alignSelf: 'center' }}
                            >
                                {setting.name}
                            </label>
                            {setting.type === 'Text-Input' && <TextInput />}
                            {setting.type === 'Number-Input' && (
                                <NumberInput
                                    id={id}
                                    max={setting.max}
                                    min={setting.min}
                                    defaultValue={setting.defaultValue}
                                />
                            )}
                            {setting.type === 'Select' && (
                                <SelectWrapper
                                    id={id}
                                    options={setting.options}
                                />
                            )}
                            {setting.type === 'Checkbox' && (
                                <Checkbox
                                    defaultValue={setting.defaultValue}
                                    id={id}
                                />
                            )}
                        </React.Fragment>
                    );
                })}
            </div>
        </div>
    );
}

function Checkbox(props: { defaultValue?: boolean; id: string }) {
    return (
        <Switch.Root
            defaultChecked={props.defaultValue ?? false}
            className="relative aspect-[1.75/1] h-full rounded-full bg-gray-200 outline outline-1 outline-gray-200 data-[state=checked]:bg-[#68eb5d]"
            id={props.id}
        >
            <Switch.Thumb className="block aspect-square h-full rounded-full bg-white   transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[75%]" />
        </Switch.Root>
    );
}

function SelectWrapper(props: {
    id: string;
    defaultValue: string;
    options: Array<{ value: string; id: string }>;
}) {
    return (
        <Select.Root>
            <Select.Trigger
                className="flex w-full items-center justify-between  data-[placeholder]:text-red-500"
                id={props.id}
            >
                <Select.Value defaultValue={props.defaultValue} />
                <Select.Icon className="text-black">
                    <CaretDown size={32} weight="duotone" />
                </Select.Icon>
            </Select.Trigger>

            <Select.Portal>
                <Select.Content className="rounded-md bg-white p-5 shadow-xl">
                    <Select.Viewport className="flex flex-col gap-2 ">
                        {props.options.map((option, index) => (
                            <Select.Item key={index} value={option.id}>
                                <Select.ItemText>
                                    {option.value}
                                </Select.ItemText>
                            </Select.Item>
                        ))}
                    </Select.Viewport>
                </Select.Content>
            </Select.Portal>
        </Select.Root>
    );
}

function NumberInput(props: {
    min: number;
    max: number;
    defaultValue?: number;
    id: string;
}) {
    let [value, setValue] = useState(props.defaultValue ?? props.min);
    return (
        <div className="flex h-full w-full gap-3">
            <Slider.Root
                className="relative flex h-full w-full touch-none select-none items-center"
                defaultValue={[props.defaultValue ?? props.min]}
                max={props.max}
                min={props.min}
                value={[value]}
                step={1}
                onValueChange={(value) => setValue(value[0])}
            >
                <Slider.Track className="relative h-[3px] grow rounded-full bg-gray-200">
                    <Slider.Range className="absolute h-full rounded-full bg-[#68eb5d]" />
                </Slider.Track>
                <Slider.Thumb className="block h-5 w-5 rounded-full bg-white shadow-md" />
            </Slider.Root>
            <input
                type="number"
                name=""
                id=""
                max={props.max}
                min={props.min}
                className="w-[7ch] rounded-md bg-gray-200 p-1 text-center"
                value={value}
                onChange={(event) =>
                    setValue(event.currentTarget.valueAsNumber)
                }
            ></input>
        </div>
    );
}

function TextInput(props: { defaultValue?: string; id: string }) {
    return (
        <input
            type="text"
            id={props.id}
            className="w-full rounded-md bg-gray-200 p-1 outline-gray-200"
        >
            {props.defaultValue}
        </input>
    );
}
