export const API_URL = import.meta.env.PROD
    ? `${window.location.origin}/api`
    : 'http://localhost:3000/api';

export interface Game {
    state: PatchworkState;
}

export interface PatchworkState {
    patches: Patch[];
    time_board: TimeBoard;
    player_1: Player;
    player_2: Player;
    turn_type: string;
    status_flags: StatusFlags;
    notation: string;
    game_id: string;
}

export interface Patch {
    id: number;
    button_cost: number;
    time_cost: number;
    button_income: number;
    tiles: number[][];
}

export interface TimeBoard {
    player_1: number;
    player_2: number;
    special_patches: number[];
    button_income_triggers: number[];
    board: Array<{
        player_1: boolean;
        player_2: boolean;
        special_patch: boolean;
        button_income_trigger: boolean;
    }>;
}

export interface Player {
    position: number;
    button_balance: number;
    quilt_board: QuiltBoard;
}

export interface QuiltBoard {
    button_income: number;
    tiles: boolean[][];
}

export interface StatusFlags {
    current_player: number;
    special_tile: null | 0 | 1; // TODO: check if 0 or 1 is correct
    first_goal: null | 0 | 1;
}
