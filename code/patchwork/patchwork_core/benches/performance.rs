use criterion::{black_box, criterion_group, criterion_main, Criterion};
use patchwork_core::{ActionId, GameOptions, Notation, Patchwork};

fn get_initial_state(c: &mut Criterion) {
    c.bench_function("get_valid_actions", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                Some(GameOptions { seed })
            },
            |args: Option<GameOptions>| black_box(Patchwork::get_initial_state(args)),
        );
    });
}

fn get_valid_actions(c: &mut Criterion) {
    c.bench_function("get_valid_actions", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                Patchwork::get_initial_state(Some(GameOptions { seed }))
            },
            |patchwork: Patchwork| black_box(patchwork.get_valid_actions()),
        );
    });
}

fn get_all_valid_actions(c: &mut Criterion) {
    let state = Patchwork::load_from_notation("000000000000000000000B5I0P0 000000000000000000000B5I0P0 0 N 8/14/19/4/5/6/7/1/9/10/11/12/13/2/15/16/17/18/3/20/21/22/23/24/25/26/27/28/29/30/31/32/0").unwrap();
    let valid_actions = state.get_valid_actions();
    println!("valid_actions length: {:?}", valid_actions.len());

    c.bench_function("get_valid_actions [all]", |b| {
        b.iter_with_setup(
            || Patchwork::load_from_notation("000000000000000000000B5I0P0 000000000000000000000B5I0P0 0 N 8/14/19/4/5/6/7/1/9/10/11/12/13/2/15/16/17/18/3/20/21/22/23/24/25/26/27/28/29/30/31/32/0").unwrap(),
            |patchwork: Patchwork| black_box(patchwork.get_valid_actions()),
        );
    });
}

fn get_random_action(c: &mut Criterion) {
    c.bench_function("get_random_action", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                Patchwork::get_initial_state(Some(GameOptions { seed }))
            },
            |patchwork: Patchwork| black_box(patchwork.get_random_action()),
        );
    });
}

fn do_action(c: &mut Criterion) {
    c.bench_function("do_action", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed }));
                for _ in 0..(seed % 22) {
                    patchwork.do_action(patchwork.get_random_action(), false).unwrap();
                }
                let action = patchwork.get_random_action();
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, ActionId)| {
                let mut patchwork = patchwork;
                black_box(patchwork.do_action(action, false))
            },
        );
    });
}

fn undo_action(c: &mut Criterion) {
    c.bench_function("undo_action", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed }));
                for _ in 0..(seed % 22) {
                    patchwork.do_action(patchwork.get_random_action(), false).unwrap();
                }
                let action = patchwork.get_random_action();
                patchwork.do_action(action, false).unwrap();
                (patchwork, action)
            },
            |(patchwork, action): (Patchwork, ActionId)| {
                let mut patchwork = patchwork;
                black_box(patchwork.undo_action(action, false))
            },
        );
    });
}

fn clone(c: &mut Criterion) {
    c.bench_function("clone", |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                Patchwork::get_initial_state(Some(GameOptions { seed }))
            },
            |patchwork: Patchwork| black_box(patchwork.clone()),
        );
    });
}

criterion_group!(
    benches,
    get_initial_state,
    get_valid_actions,
    get_all_valid_actions,
    get_random_action,
    do_action,
    undo_action,
    clone
);
criterion_main!(benches);
