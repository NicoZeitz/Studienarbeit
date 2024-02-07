import { CSSProperties } from 'react';
import { Patch } from '../../constants.ts';

export interface PatchProps {
    patch: Patch;
    rotation: 0 | 1 | 2 | 3;
    flipped: boolean;
}

const patchAttribute = [
    [],
    [],
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
        { gridRow: '1 / span 4', gridColumn: '2 / span 2' },
        { gridRow: '1 / span 4', gridColumn: '3 / span 2', rotate: '90deg' },
        { gridRow: '2 / span 4', gridColumn: '2 / span 2', rotate: '180deg' },
        { gridRow: '1 / span 4', gridColumn: '2 / span 2', rotate: '270deg' },
    ],
] as const satisfies Array<Array<CSSProperties>>;

export default function Patch(props: PatchProps) {
    const { patch, rotation, flipped } = props;
    const attributeIndex = rotation + (flipped ? 4 : 0);

    const src = `/assets/patch/${patch.id.toString().padStart(2, '0')}-front.avif`;

    // TODO: percentages for grid size
    return (
        <div className="grid h-36 max-h-36 w-36 max-w-36 grid-cols-5 grid-rows-5 ">
            <picture
                style={patchAttribute[patch.id][attributeIndex]}
                className="h-full w-full object-contain drop-shadow-md"
            >
                <source src={src} srcSet={src} />
                <img
                    className="h-full w-full object-contain"
                    src={src}
                    alt={`Patch ${patch.id}`}
                />
            </picture>
        </div>
    );
}
