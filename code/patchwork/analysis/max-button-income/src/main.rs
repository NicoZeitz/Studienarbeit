use itertools::Itertools;
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Patch {
    index: usize,
    tiles: usize,
    buttons: usize,
    time_cost: usize,
}

#[rustfmt::skip]
fn main() {
    let data  = [
        Patch { index: 27, tiles: 5, buttons: 1, time_cost: 1 },
        Patch { index: 13, tiles: 6, buttons: 2, time_cost: 2 },
        Patch { index:  1, tiles: 5, buttons: 3, time_cost: 4 },
        Patch { index: 18, tiles: 5, buttons: 2, time_cost: 3 },
        Patch { index:  4, tiles: 4, buttons: 3, time_cost: 6 },
        Patch { index: 19, tiles: 4, buttons: 1, time_cost: 2 },
        Patch { index: 26, tiles: 4, buttons: 1, time_cost: 2 },
        Patch { index:  9, tiles: 4, buttons: 2, time_cost: 5 },
        Patch { index: 12, tiles: 6, buttons: 3, time_cost: 5 },
        Patch { index: 17, tiles: 5, buttons: 2, time_cost: 4 },
        Patch { index:  3, tiles: 6, buttons: 3, time_cost: 6 },
        Patch { index: 14, tiles: 4, buttons: 2, time_cost: 6 },
        Patch { index: 15, tiles: 6, buttons: 2, time_cost: 4 },
        Patch { index: 28, tiles: 4, buttons: 1, time_cost: 3 },
        Patch { index: 29, tiles: 5, buttons: 2, time_cost: 5 },
        Patch { index: 10, tiles: 5, buttons: 1, time_cost: 3 },
        Patch { index: 30, tiles: 6, buttons: 2, time_cost: 6 },
        Patch { index: 32, tiles: 6, buttons: 1, time_cost: 3 },
        Patch { index: 31, tiles: 5, buttons: 1, time_cost: 4 },
        Patch { index:  2, tiles: 8, buttons: 1, time_cost: 3 },
        Patch { index: 20, tiles: 7, buttons: 1, time_cost: 4 },
        Patch { index: 16, tiles: 6, buttons: 1, time_cost: 5 },
    ];

    let best_permutation = std::sync::Arc::new(std::sync::Mutex::new(Vec::<usize>::new()));
    let best_income = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let iteration = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let target: u128 = (1u128..=data.len() as u128).product();

    data.iter().permutations(data.len()).par_bridge().for_each(|permutation| {
        let mut current_max_time_cost = 0;
        let mut current_tiles = 0;
        let mut current_time_cost = 0;
        let mut current_income = 0;
        let mut current_permutation = vec![];

        for patch in permutation {
            if patch.tiles + current_tiles > 81 {
                continue;
            }

            let new_time_cost = current_time_cost + patch.time_cost;
            let new_max_time_cost = current_max_time_cost.max(patch.time_cost);
            if new_time_cost - new_max_time_cost > 53 {
                continue;
            }

            current_tiles += patch.tiles;
            current_time_cost += patch.time_cost;
            current_max_time_cost = current_max_time_cost.max(patch.time_cost);
            current_income += patch.buttons;
            current_permutation.push(patch.index);

            if current_tiles == 81 {
                break;
            }
        }

        if current_income > best_income.fetch_max(current_income, std::sync::atomic::Ordering::Relaxed) {
            println!("Best income: {}", current_income);
            println!("Best permutation: {:?}", current_permutation);

            let mut guard = best_permutation.lock().unwrap();
            *guard = current_permutation;

        }

        let iter = iteration.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if (iter +1) % 1_000_000 == 0 {
            println!("{:?} out of {:?}", iter + 1, target);
        }
    });

    println!("Best income: {}", best_income.load(std::sync::atomic::Ordering::SeqCst));
    println!("Best permutation: {:?}", best_permutation.lock().unwrap());
}
