use candle_core::{DType, Device, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use evaluator::NeuralNetworkEvaluator;
use patchwork_core::{ActionId, Patchwork};

pub fn main() {
    let vm = VarMap::new();
    let vb = VarBuilder::from_varmap(&vm, DType::F32, &Device::Cpu);
    let model = NeuralNetworkEvaluator::new(vb.pp("neural_network_evaluator")).unwrap();
    let mut opt = AdamW::new(vm.all_vars(), ParamsAdamW::default()).unwrap();

    let state = Patchwork::get_initial_state(None);
    let mut state_2 = state.clone();
    state_2.do_action(ActionId::walking(0), false).unwrap();

    for step in 0..10000 {
        if step % 2 == 0 {
            let score = model.forward(&state).unsqueeze(0).unwrap();
            let one_tensor = Tensor::from_slice(&[1f32], (1,), &Device::Cpu).unwrap();
            let loss = candle_nn::loss::binary_cross_entropy_with_logit(&score, &one_tensor).unwrap();
            opt.backward_step(&loss).unwrap();
            println!(
                "{step} {} {}",
                score.squeeze(0).unwrap().to_vec0::<f32>().unwrap(),
                loss.to_vec0::<f32>().unwrap()
            );
        } else {
            let score = model.forward(&state_2).unsqueeze(0).unwrap();
            let one_tensor = Tensor::from_slice(&[-1f32], (1,), &Device::Cpu).unwrap();
            let loss = candle_nn::loss::binary_cross_entropy_with_logit(&score, &one_tensor).unwrap();
            opt.backward_step(&loss).unwrap();
            println!(
                "{step} {} {}",
                score.squeeze(0).unwrap().to_vec0::<f32>().unwrap(),
                loss.to_vec0::<f32>().unwrap()
            );
        }
    }

    // test_full(&vb);
    // println!("===============================");
    // test_full(&vb);
}

// fn test_full(vb: &VarBuilder) {
//     println!("========= FULL ========");
//     state.do_action(ActionId::walking(0), false).unwrap();

//     let now = std::time::Instant::now();
//     let other_eval = println!("           NEW DONE: {: >8.2?}", now.elapsed());

//     let now = std::time::Instant::now();
//     let res = other_eval.evaluate_intermediate_node(&state);
//     println!("OTHER EVALUATE DONE: {: >8.2?}, RESULT: {:?}", now.elapsed(), res);
// }
