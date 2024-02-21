import PlayerSelector from '../PlayerSelector/PlayerSelector';

export interface MainPageProps {}

export default function MainPage(props: MainPageProps) {
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
                <PlayerSelector label="Spieler 1 (grün)" player={1} />
                <div className="w-[20dvw] max-w-[50ch]"></div>
                <PlayerSelector label="Spieler 2 (gelb)" player={2} />
            </div>

            <button className="rounded-lg bg-[#68eb5d] px-10 py-3 text-lg font-medium">
                Spiel starten
            </button>
        </div>
    );
}
