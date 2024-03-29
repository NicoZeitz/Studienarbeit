use patchwork_core::{ActionId, PatchManager, QuiltBoard};

fn main() {
    // let action_ids = [
    //     ActionId::from_bits(44934),
    //     ActionId::from_bits(45833),
    //     ActionId::from_bits(46365),
    //     ActionId::from_bits(47202),
    //     ActionId::from_bits(49165),
    //     ActionId::from_bits(49579),
    //     ActionId::from_bits(49916),
    //     ActionId::from_bits(51744),
    //     ActionId::from_bits(52143),
    //     ActionId::from_bits(53463),
    //     ActionId::from_bits(54397),
    //     ActionId::from_bits(56191),
    //     ActionId::from_bits(57963),
    //     ActionId::from_bits(58953),
    // ];

    let patch_ids = [ 1,  3,  4,  6, 10, 11, 12, 16, 17, 20, 22, 26, 30, 32];

    let action_ids = brute_force(patch_ids);
    verify(action_ids);
}

#[allow(dead_code)]
fn verify(action_ids: [ActionId; 14]) {
    let actions = action_ids.map(|action_id| action_id.to_action());

    let mut quilt_board = QuiltBoard::default();
    for action_id in action_ids {
        quilt_board.do_action(action_id);
    }
    println!("Quilt Board:");
    println!("{quilt_board}");

    for action in actions {
        let mut board = QuiltBoard::default();
        board.do_action(action.to_surrogate_action_id());

        println!("============================================");
        println!("Patch Id: {:?}", action.try_get_patch_id().unwrap());
        println!(
            "Patch: {:?}",
            PatchManager::get_tiles(action.try_get_patch_id().unwrap())
        );
        println!("Board: ");
        println!("{board}");
        println!("Row: {}", action.try_get_row().unwrap());
        println!("Column: {}", action.try_get_column().unwrap());
        println!("Rotation: {}", action.try_get_rotation().unwrap());
        println!("Orientation: {}", action.try_get_orientation().unwrap());
        println!(
            "Patch Transformation Index: {}",
            action.try_get_patch_transformation_index().unwrap()
        );
    }
}

#[allow(dead_code)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn brute_force(patch_ids: [u8; 14]) -> [ActionId; 14] {
    let patches = patch_ids.map(PatchManager::get_patch);

    let mut current_iteration = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut current_action = [
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
        ActionId::null(),
    ];
    println!(
        "Patches: {:?}",
        patches.map(|patch| patch.id).iter().collect::<Vec<_>>(),
    );

    let board = QuiltBoard::default();

    current_iteration[0] += 1;
    println!("Iterations: {current_iteration:?}");
    let valid_actions = board.get_valid_actions_for_patch(patches[0], 0, true);
    if valid_actions.is_empty() {
        return current_action;
    }

    for valid_action in valid_actions {
        current_action[0] = valid_action;
        let mut board = board.clone();
        board.do_action(valid_action);

        current_iteration[1] += 1;
        println!("Iterations: {current_iteration:?}");
        let valid_actions = board.get_valid_actions_for_patch(patches[1], 0, true);
        if valid_actions.is_empty() {
            continue;
        }

        for valid_action in valid_actions {
            current_action[1] = valid_action;
            let mut board = board.clone();
            board.do_action(valid_action);

            current_iteration[2] += 1;
            println!("Iterations: {current_iteration:?}");
            let valid_actions = board.get_valid_actions_for_patch(patches[2], 0, true);
            if valid_actions.is_empty() {
                continue;
            }

            for valid_action in valid_actions {
                current_action[2] = valid_action;
                let mut board = board.clone();
                board.do_action(valid_action);

                current_iteration[3] += 1;
                println!("Iterations: {current_iteration:?}");
                let valid_actions = board.get_valid_actions_for_patch(patches[3], 0, true);
                if valid_actions.is_empty() {
                    continue;
                }

                for valid_action in valid_actions {
                    current_action[3] = valid_action;
                    let mut board = board.clone();
                    board.do_action(valid_action);

                    current_iteration[4] += 1;
                    println!("Iterations: {current_iteration:?}");
                    let valid_actions = board.get_valid_actions_for_patch(patches[4], 0, true);
                    if valid_actions.is_empty() {
                        continue;
                    }

                    for valid_action in valid_actions {
                        current_action[4] = valid_action;
                        let mut board = board.clone();
                        board.do_action(valid_action);

                        current_iteration[5] += 1;
                        let valid_actions = board.get_valid_actions_for_patch(patches[5], 0, true);
                        if valid_actions.is_empty() {
                            continue;
                        }

                        for valid_action in valid_actions {
                            current_action[5] = valid_action;
                            let mut board = board.clone();
                            board.do_action(valid_action);

                            current_iteration[6] += 1;
                            let valid_actions = board.get_valid_actions_for_patch(patches[6], 0, true);
                            if valid_actions.is_empty() {
                                continue;
                            }

                            for valid_action in valid_actions {
                                current_action[6] = valid_action;
                                let mut board = board.clone();
                                board.do_action(valid_action);

                                current_iteration[7] += 1;
                                let valid_actions = board.get_valid_actions_for_patch(patches[7], 0, true);
                                if valid_actions.is_empty() {
                                    continue;
                                }

                                for valid_action in valid_actions {
                                    current_action[7] = valid_action;
                                    let mut board = board.clone();
                                    board.do_action(valid_action);

                                    current_iteration[8] += 1;
                                    let valid_actions = board.get_valid_actions_for_patch(patches[8], 0, true);
                                    if valid_actions.is_empty() {
                                        continue;
                                    }

                                    for valid_action in valid_actions {
                                        current_action[8] = valid_action;
                                        let mut board = board.clone();
                                        board.do_action(valid_action);

                                        current_iteration[9] += 1;
                                        let valid_actions = board.get_valid_actions_for_patch(patches[9], 0, true);
                                        if valid_actions.is_empty() {
                                            continue;
                                        }

                                        for valid_action in valid_actions {
                                            current_action[9] = valid_action;
                                            let mut board = board.clone();
                                            board.do_action(valid_action);

                                            current_iteration[10] += 1;
                                            let valid_actions = board.get_valid_actions_for_patch(patches[10], 0, true);
                                            if valid_actions.is_empty() {
                                                continue;
                                            }

                                            for valid_action in valid_actions {
                                                current_action[10] = valid_action;
                                                let mut board = board.clone();
                                                board.do_action(valid_action);

                                                current_iteration[11] += 1;
                                                let valid_actions =
                                                    board.get_valid_actions_for_patch(patches[11], 0, true);
                                                if valid_actions.is_empty() {
                                                    continue;
                                                }

                                                for valid_action in valid_actions {
                                                    current_action[11] = valid_action;
                                                    let mut board = board.clone();
                                                    board.do_action(valid_action);

                                                    current_iteration[12] += 1;
                                                    let valid_actions =
                                                        board.get_valid_actions_for_patch(patches[12], 0, true);
                                                    if valid_actions.is_empty() {
                                                        continue;
                                                    }

                                                    for valid_action in valid_actions {
                                                        current_action[12] = valid_action;
                                                        let mut board = board.clone();
                                                        board.do_action(valid_action);

                                                        current_iteration[13] += 1;
                                                        let valid_actions =
                                                            board.get_valid_actions_for_patch(patches[13], 0, true);
                                                        if valid_actions.is_empty() {
                                                            continue;
                                                        }

                                                        if valid_actions.is_empty() {
                                                            continue;
                                                        }

                                                        let valid_action = valid_actions[0];

                                                        current_action[13] = valid_action;
                                                        let mut board = board.clone();
                                                        board.do_action(valid_action);

                                                        println!("Found solution!");
                                                        println!("Transforms: {current_action:?}");
                                                        return current_action;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("No solution found");
    current_action
}
