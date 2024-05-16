import { CSSProperties } from 'react';
import { Patch } from '../../constants.ts';
import { motion } from 'framer-motion';

export interface PatchProps {
    patch: Patch;
    rotation: 0 | 1 | 2 | 3;
    flipped: boolean;
}

const patchAttribute = [
    [
        // patch id 00
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '0deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '90deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '180deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '270deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '90deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '180deg' },
        { gridRow: '5 / span 2', gridColumn: '4 / span 4', rotate: '270deg' },
    ],
    [
        // patch id 01
        { gridRow: '4 / span 6', gridColumn: '4 / span 6' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '90deg' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '180deg' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '270deg' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '90deg' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '180deg' },
        { gridRow: '4 / span 6', gridColumn: '4 / span 6', rotate: '270deg' },
    ],
    [
        // patch id 02
        { gridRow: '3 / span 6', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 03
        { gridRow: '3 / span 6', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 04
        { gridRow: '4 / span 4', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 05
        { gridRow: '2 / span 8', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 06
        { gridRow: '2 / span 8', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 07
        { gridRow: '3 / span 6', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 08
        { gridRow: '3 / span 6', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 09
        { gridRow: '4 / span 4', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 10
        { gridRow: '2 / span 8', gridColumn: '4 / span 4' },
        { gridRow: '2 / span 8', gridColumn: '6 / span 4', rotate: '90deg' },
        { gridRow: '4 / span 8', gridColumn: '4 / span 4', rotate: '180deg' },
        { gridRow: '2 / span 8', gridColumn: '4 / span 4', rotate: '270deg' },
    ],
    [
        // patch id 11
        { gridRow: '3 / span 6', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 12
        { gridRow: '2 / span 8', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 13
        { gridRow: '2 / span 8', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 14
        { gridRow: '3 / span 6', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 15
        { gridRow: '4 / span 4', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 16
        { gridRow: '2 / span 8', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 17
        { gridRow: '3 / span 6', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 18
        { gridRow: '4 / span 4', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 19
        { gridRow: '4 / span 4', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 20
        { gridRow: '3 / span 6', gridColumn: '1 / span 10' },
        { gridRow: '3 / span 10', gridColumn: '1 / span 20', rotate: '90deg' },
    ],
    [
        // patch id 21
        { gridRow: '4 / span 4', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 22
        { gridRow: '4 / span 4', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 23
        { gridRow: '4 / span 4', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 24
        { gridRow: '3 / span 6', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 25
        { gridRow: '5 / span 2', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 26
        { gridRow: '3 / span 6', gridColumn: '4 / span 4' },
    ],
    [
        // patch id 27
        { gridRow: '5 / span 2', gridColumn: '1 / span 10' },
    ],
    [
        // patch id 28
        { gridRow: '5 / span 2', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 29
        { gridRow: '3 / span 6', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 30
        { gridRow: '3 / span 6', gridColumn: '3 / span 6' },
    ],
    [
        // patch id 31
        { gridRow: '4 / span 4', gridColumn: '2 / span 8' },
    ],
    [
        // patch id 32
        { gridRow: '3 / span 6', gridColumn: '2 / span 8' },
    ],
] as const satisfies Array<Array<CSSProperties>>;

export default function Patch(propss: PatchProps) {
    const props = { ...propss };
    if (props.patch.id == 0) {
        props.rotation = 1;
        props.flipped = false;
        props.patch.id = 0;
    }

    const { patch, rotation, flipped } = props;
    const attributeIndex = rotation + (flipped ? 4 : 0);
    const side = flipped ? 'back' : 'front';
    const src = `/assets/patch/${patch.id.toString().padStart(2, '0')}-${side}.avif`;

    // TODO: percentages for grid size
    return (
        <div className="grid h-[220px] w-[220px] grid-cols-10 grid-rows-10 ">
            <motion.picture
                style={{
                    ...patchAttribute[patch.id][attributeIndex],
                    transitionProperty: 'rotation',
                }}
                className="h-full w-full object-contain drop-shadow-md duration-300"
            >
                <source src={src} srcSet={src} />
                <img
                    className="h-full w-full object-contain"
                    src={src}
                    alt={`Patch ${patch.id}`}
                />
            </motion.picture>
        </div>
    );
}
