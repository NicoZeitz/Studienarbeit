use action_orderer::{ActionList, ActionOrderer, TableActionOrderer};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use patchwork_core::{GameOptions, Patchwork};

#[allow(clippy::unit_arg)]
fn table_action_orderer<T>(c: &mut Criterion)
where
    T: ActionOrderer + Default,
{
    let typename_t = std::any::type_name::<T>().split("::").last().unwrap();
    let bench_name = format!("{}::score_action", typename_t);

    c.bench_function(&bench_name, |b| {
        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                let state = Patchwork::get_initial_state(Some(GameOptions { seed }));
                let actions = state.get_valid_actions();
                let scores = vec![0.0; actions.len()];
                (actions, scores)
            },
            |(mut actions, mut scores)| {
                let mut action_list: ActionList<'_> = ActionList::new(&mut actions, &mut scores);
                let table_action_orderer = T::default();
                black_box(table_action_orderer.score_actions(&mut action_list, None, 42))
            },
        );
    });
}

criterion_group!(benches, table_action_orderer<TableActionOrderer>);
criterion_main!(benches);
