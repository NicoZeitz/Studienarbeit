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
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [],
    [
        // patch id 10
        { gridRow: '2 / span 8', gridColumn: '4 / span 4' },
        { gridRow: '2 / span 8', gridColumn: '6 / span 4', rotate: '90deg' },
        { gridRow: '4 / span 8', gridColumn: '4 / span 4', rotate: '180deg' },
        { gridRow: '2 / span 8', gridColumn: '4 / span 4', rotate: '270deg' },
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
        <div className="grid h-36 max-h-36 w-36 max-w-36 grid-cols-10 grid-rows-10 ">
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
