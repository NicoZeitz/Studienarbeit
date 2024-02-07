import { useEffect, useState } from 'react';
import TimeBoard from '../TimeBoard/TimeBoard.tsx';
import QuiltBoard from '../QuiltBoard/QuiltBoard.tsx';
import { useLocation } from 'react-router-dom';
import { motion } from 'framer-motion';
// import { StateContext } from '../GameLayout/GameLayout.tsx';

export interface TableCenterProps {}

export default function TableCenter(props: TableCenterProps) {
    // const state = useContext(StateContext);
    const location = useLocation();
    const [pos1, setPos1] = useState(0);
    const [pos2, setPos2] = useState(30);

    useEffect(() => {
        const interval = setInterval(() => {
            setPos1((pos) => (pos + 1) % 54);
            setPos2((pos) => (pos + 2) % 54);
        }, 250);

        return () => clearInterval(interval);
    });

    return (
        <motion.main
            initial={{ y: '-50vw' }}
            animate={{ y: 0 }}
            transition={{
                duration: 0.75,
                delay: 0.5,
                type: 'spring',
                bounce: 0.3,
            }}
            className="grid grid-rows-1 gap-[0.5%]"
            style={{ gridTemplateColumns: '30% 1fr 30%' }}
        >
            <motion.div
                initial={{ x: '25%', scale: 0.75 }}
                animate={{ x: 0, scale: 1 }}
                transition={{
                    duration: 0.75,
                    delay: 0.75,
                    type: 'spring',
                    bounce: 0.3,
                }}
                className="col-span-1 col-start-1 row-span-1 row-start-1 grid items-center justify-items-end"
            >
                <QuiltBoard player={1} />
            </motion.div>
            <motion.div
                initial={{ x: '-25%', scale: 0.75 }}
                animate={{ x: 0, scale: 1 }}
                transition={{
                    duration: 0.75,
                    delay: 0.75,
                    type: 'spring',
                    bounce: 0.3,
                }}
                className="col-span-1 col-start-3 row-span-1 row-start-1 grid items-center justify-items-start"
            >
                <QuiltBoard player={2} />
            </motion.div>
            <div className="relative col-span-1 col-start-2 row-span-1 row-start-1">
                <TimeBoard
                    player1Position={pos1}
                    player2Position={pos2}
                    player={1}
                />
            </div>
        </motion.main>
    );
}
