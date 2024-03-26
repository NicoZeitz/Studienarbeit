export type PlayerSetting =
    | {
          id: string;
          name: string;
          type: 'Text-Input';
          defaultValue?: string;
      }
    | {
          id: string;
          name: string;
          type: 'Select';
          defaultValue?: string;
          options: Array<{ value: string; id: string }>;
      }
    | {
          id: string;
          name: string;
          type: 'Checkbox';
          defaultValue: boolean;
      }
    | {
          id: string;
          name: string;
          type: 'Number-Input';
          min: number;
          max: number;
          defaultValue?: number;
      };

export type PlayerSettings = {
    [id in keyof typeof playerSettings]: {
        [id: string]: string | number | boolean | undefined;
    };
};

export const playerSettings = {
    Mensch: [
        {
            id: 'human-name',
            name: 'Name',
            type: 'Text-Input',
        },
    ],
    'KI-Random': [
        {
            id: 'random-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'random-seed',
            name: 'Seed',
            type: 'Number-Input',
            min: 0,
            max: 1000,
            defaultValue: (Math.random() * 1000 + 1) | 0,
        },
    ],
    'KI-Greedy': [
        {
            id: 'greedy-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'greedy-evaluator',
            name: 'Evaluierer',
            type: 'Select',
            defaultValue: 'static',
            options: [
                { value: 'Statischer Evaluierer', id: 'static' },
                { value: 'Zufallsspiel', id: 'win' },
                { value: 'Bewertetes Zufallsspiel', id: 'score' },
                { value: 'Neuronales Netz', id: 'nn' },
            ],
        },
    ],
    'KI-Minimax': [
        {
            id: 'minimax-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'minimax-depth',
            name: 'Tiefe',
            type: 'Number-Input',
            min: 1,
            max: 20,
            defaultValue: 10,
        },
        {
            id: 'minimax-branching-factor',
            name: 'Flickenzüge',
            type: 'Number-Input',
            min: 1,
            max: 10,
            defaultValue: 3,
        },
    ],
    'KI-PVS': [
        {
            id: 'pvs-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'pvs-time',
            name: 'Zugzeit in Sekunden',
            type: 'Number-Input',
            min: 1,
            max: 60,
            defaultValue: 10,
        },
        {
            id: 'pvs-evaluator',
            name: 'Evaluierer',
            type: 'Select',
            defaultValue: 'static',
            options: [
                { value: 'Statischer Evaluierer', id: 'static' },
                { value: 'Zufallsspiel', id: 'win' },
                { value: 'Bewertetes Zufallsspiel', id: 'score' },
                { value: 'Neuronales Netz', id: 'nn' },
            ],
        },
        {
            id: 'pvs-failing-strategy',
            name: 'Failing strategy',
            type: 'Select',
            options: [
                { value: 'Soft', id: 'soft' },
                { value: 'Hard', id: 'hard' },
            ],
        },
        {
            id: 'pvs-aspiration-window',
            name: 'Aspiration window',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'pvs-late-move-reduction',
            name: 'Late Move Reduction',
            type: 'Checkbox',
            defaultValue: true,
        },
        {
            id: 'pvs-late-move-pruning',
            name: 'Late Move Pruning',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'pvs-transposition-table',
            name: 'Transposition Table',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'pvs-lazy-smp',
            name: 'Lazy-SMP',
            type: 'Checkbox',
            defaultValue: false,
        },
    ],
    'KI-MCTS': [
        {
            id: 'mcts-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'mcts-time',
            name: 'Zugzeit in Sekunden',
            type: 'Number-Input',
            min: 1,
            max: 60,
        },
        {
            id: 'mcts-tree-reuse',
            name: 'Baum wiederverwenden',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'mcts-root-parallelization',
            name: 'Wurzeln parallelisieren',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'mcts-leaf-parallelization',
            name: 'Blätter parallelisieren',
            type: 'Checkbox',
            defaultValue: false,
        },
        {
            id: 'mcts-tree-policy',
            name: 'Tree Policy',
            type: 'Select',
            options: [
                { value: 'UCT', id: 'uct' },
                { value: 'Partial Score', id: 'partial-score' },
                { value: 'Score', id: 'score' },
            ],
        },
        {
            id: 'mcts-rollout-policy',
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
    'KI-AlphaZero': [
        {
            id: 'alphazero-name',
            name: 'Name',
            type: 'Text-Input',
        },
        {
            id: 'alphazero-time',
            name: 'Zugzeit in Sekunden',
            type: 'Number-Input',
            min: 1,
            max: 60,
            defaultValue: 10,
        },
    ],
} as const satisfies { [key: string]: Array<PlayerSetting> };
