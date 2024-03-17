use candle_core::{DType, Device};
use candle_nn::{VarBuilder, VarMap};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evaluator::{NeuralNetworkEvaluator, StaticEvaluator};
use patchwork_core::{Evaluator, GameOptions, Patchwork};

fn static_evaluator_forward(c: &mut Criterion) {
    c.bench_function("static_evaluator_forward", |b| {
        let evaluator = StaticEvaluator;

        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed }));

                for _ in 0..(seed % 22) {
                    patchwork.do_action(patchwork.get_random_action(), false).unwrap();
                }

                patchwork
            },
            |patchwork: Patchwork| black_box(evaluator.evaluate_node(&patchwork)),
        );
    });
}

fn neural_network_evaluator_forward(c: &mut Criterion) {
    c.bench_function("neural_network_evaluator_forward", |b| {
        let var_map = VarMap::new();
        let var_builder = VarBuilder::from_varmap(&var_map, DType::F32, &Device::Cpu);
        let neural_network_evaluator = NeuralNetworkEvaluator::new(var_builder).unwrap();

        b.iter_with_setup(
            || {
                let seed = rand::random::<u64>();
                let mut patchwork = Patchwork::get_initial_state(Some(GameOptions { seed }));

                for _ in 0..(seed % 22) {
                    patchwork.do_action(patchwork.get_random_action(), false).unwrap();
                }

                patchwork
            },
            |patchwork: Patchwork| black_box(neural_network_evaluator.evaluate_node(&patchwork)),
        );
    });
}

criterion_group!(benches, static_evaluator_forward, neural_network_evaluator_forward);
criterion_main!(benches);
