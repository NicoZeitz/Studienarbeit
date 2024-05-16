import { useState } from 'react';
import DuckButton from '../DuckButton/DuckButton';
import PlayerSelector from '../PlayerSelector/PlayerSelector';
import type { PlayerSettings } from '../PopupSettings/playerSettings.ts';
import { PlayerIds } from '../../player/playerIds.ts';
import { Link } from 'react-router-dom';

export interface MainPageProps {}

const defaultPlayerSettings = {
    selectedPlayer: null,
    settings: {
        Mensch: {},
        'KI-Random': {},
        'KI-Greedy': {},
        'KI-Minimax': {},
        'KI-PVS': {},
        'KI-MCTS': {},
        'KI-AlphaZero': {},
    },
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export default function MainPage(props: MainPageProps) {
    const [player1, setPlayer1] = useState<{
        selectedPlayer: null | PlayerIds;
        settings: PlayerSettings;
    }>(defaultPlayerSettings);
    const [player2, setPlayer2] = useState<{
        selectedPlayer: null | PlayerIds;
        settings: PlayerSettings;
    }>(defaultPlayerSettings);

    return (
        <div className="flex h-[100dvh]  flex-col items-center justify-center">
            <h1 className="mb-5 text-4xl">Studienarbeit DHBW Karlsruhe</h1>
            <h2 className="mb-3 max-w-[60ch] text-center text-2xl">
                <q style={{ quotes: '"„" "“" "‚" "‘"' }}>
                    Mathematische Analyse und prototypische Implementierung
                    einer geeigneten Computerspielengine mithilfe maschinellen
                    Lernens für das Brettspiel Patchwork
                </q>
            </h2>

            <h2 className="mb-24 text-center text-2xl">
                von Fabian Wolf und Nico Zeitz
            </h2>

            <div className="mb-20 flex">
                <PlayerSelector
                    label="Spieler 1 (grün)"
                    playerNumber={1}
                    player={player1}
                />
                <div className="w-[20dvw] max-w-[50ch]"></div>
                <PlayerSelector
                    label="Spieler 2 (gelb)"
                    playerNumber={2}
                    player={player2}
                />
            </div>
            <Link to="/game/c793dbe7-5829-428a-a249-55a03eb091c9">
                <button className="rounded-lg bg-[#68eb5d] px-10 py-3 text-lg font-medium">
                    Spiel starten
                </button>
            </Link>
            <DuckButton />
        </div>
    );
}
