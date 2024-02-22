import { motion } from 'framer-motion';
import { useState } from 'react';

export default function DuckButton() {
    const [allDucks, setAllDucks] = useState<Map<string, number[]>>(new Map());

    const rain = () => {
        setAllDucks((prevDucks) => {
            prevDucks.set(
                Math.random().toString(),
                Array.from({ length: 150 }, () => Math.random()),
            );
            return new Map(prevDucks);
        });
        setTimeout(() => {
            const audio = new Audio('assets/duck.mp3');
            audio.play();
        }, 500);
    };

    return (
        <>
            <button onClick={() => rain()} className="fixed bottom-0 right-0">
                🦆
            </button>

            {[...allDucks.entries()].map(([id, ducks]) => {
                return (
                    <>
                        {ducks.map((duck, index) => (
                            <motion.span
                                key={duck}
                                initial={{
                                    translateY: '-7dvh',
                                    translateX: `${Math.random() * 100}dvw`,
                                }}
                                animate={{ translateY: '110dvh' }}
                                transition={{
                                    duration: 2,
                                    delay: index * 0.015,
                                    type: 'tween',
                                    ease: 'backIn',
                                }}
                                onAnimationComplete={() =>
                                    setAllDucks((prevDucks) => {
                                        prevDucks.get(id)?.splice(index, 1);
                                        if (prevDucks.get(id)?.length === 0) {
                                            prevDucks.delete(id);
                                        }
                                        return new Map(prevDucks);
                                    })
                                }
                                className="fixed left-0 top-0"
                                style={{
                                    scale: duck * 6,
                                }}
                            >
                                🦆
                            </motion.span>
                        ))}
                    </>
                );
            })}
        </>
    );
}
