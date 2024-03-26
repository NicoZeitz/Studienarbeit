import * as Slider from '@radix-ui/react-slider';
import * as Switch from '@radix-ui/react-switch';
import * as Select from '@radix-ui/react-select';
import React from 'react';
import { CaretDown } from '@phosphor-icons/react';
import { playerSettings } from './playerSettings.ts';
import type { SettingsState } from '../PlayerSelector/PlayerSelector.tsx';

export interface PopupSettingsProps {
    player: 1 | 2;
    playerType: keyof typeof playerSettings;
    settingsState: SettingsState;
    setSettingsState: React.Dispatch<React.SetStateAction<SettingsState>>;
}

export default function PopupSettings(props: PopupSettingsProps) {
    return (
        <div>
            <h1 className="mb-2 font-bold">Einstellungen</h1>
            <div className="grid grid-cols-[minmax(0,1fr)_30ch] place-items-end gap-x-4 gap-y-2">
                {playerSettings[props.playerType].map((setting) => {
                    const id = `${setting.id}-${props.player}`;
                    const value =
                        props.settingsState[props.playerType][setting.id] ??
                        (
                            setting as {
                                defaultValue:
                                    | string
                                    | number
                                    | boolean
                                    | undefined;
                            }
                        ).defaultValue;
                    const callback = (value: string | number | boolean) =>
                        props.setSettingsState((state) => ({
                            ...state,
                            [props.playerType]: {
                                ...state[props.playerType],
                                [setting.id]: value,
                            },
                        }));

                    return (
                        <React.Fragment key={id}>
                            <label
                                htmlFor={id}
                                className="place-self-start self-center"
                            >
                                {setting.name}
                            </label>
                            {setting.type === 'Text-Input' && (
                                <TextInput
                                    id={id}
                                    value={value as string | undefined}
                                    onValueChanged={callback}
                                />
                            )}
                            {setting.type === 'Number-Input' && (
                                <NumberInput
                                    id={id}
                                    max={setting.max}
                                    min={setting.min}
                                    value={
                                        (value as number | undefined) ??
                                        setting.min
                                    }
                                    onValueChanged={callback}
                                />
                            )}
                            {setting.type === 'Select' && (
                                <SelectWrapper
                                    id={id}
                                    value={value as string | undefined}
                                    options={setting.options}
                                    onValueChanged={callback}
                                />
                            )}
                            {setting.type === 'Checkbox' && (
                                <Checkbox
                                    id={id}
                                    value={value as boolean | undefined}
                                    onCheckedChange={callback}
                                />
                            )}
                        </React.Fragment>
                    );
                })}
            </div>
        </div>
    );
}

function Checkbox(props: {
    id: string;
    value: boolean | undefined;
    onCheckedChange: (checked: boolean) => void;
}) {
    return (
        <Switch.Root
            defaultChecked={props.value ?? false}
            checked={props.value ?? false}
            onCheckedChange={props.onCheckedChange}
            className="relative aspect-[1.75/1] h-full rounded-full bg-gray-200 outline outline-1 outline-gray-200 data-[state=checked]:bg-[#68eb5d]"
            id={props.id}
        >
            <Switch.Thumb className="block aspect-square h-full rounded-full bg-white   transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[75%]" />
        </Switch.Root>
    );
}

function SelectWrapper(props: {
    id: string;
    value: string | undefined;
    options: Array<{ value: string; id: string }>;
    onValueChanged: (valueId: string) => void;
}) {
    return (
        <Select.Root
            onValueChange={(value) => props.onValueChanged(value)}
            value={props.value}
        >
            <Select.Trigger
                className="flex w-full items-center justify-between  data-[placeholder]:text-red-500"
                id={props.id}
            >
                <Select.Value defaultValue={props.value} />
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
    id: string;
    value: number;
    min: number;
    max: number;
    onValueChanged: (value: number) => void;
}) {
    return (
        <div className="flex h-full w-full gap-3">
            <Slider.Root
                className="relative flex h-full w-full touch-none select-none items-center"
                defaultValue={[props.value]}
                max={props.max}
                min={props.min}
                value={[props.value]}
                step={1}
                onValueChange={(value) => props.onValueChanged(value[0])}
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
                value={props.value}
                onChange={(event) =>
                    props.onValueChanged(event.currentTarget.valueAsNumber)
                }
            ></input>
        </div>
    );
}

function TextInput(props: {
    id: string;
    value: string | undefined;
    onValueChanged: (value: string) => void;
}) {
    return (
        <input
            type="text"
            id={props.id}
            onChange={(event) =>
                props.onValueChanged(event.currentTarget.value)
            }
            className="w-full rounded-md bg-gray-200 p-1 outline-gray-200"
            value={props.value ?? ''}
        />
    );
}
