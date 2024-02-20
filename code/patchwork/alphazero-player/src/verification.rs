use alphazero_player::PatchZero;
use candle_core::{DType, Device, IndexOp, Result};
use candle_nn::{VarBuilder, VarMap};

fn main() -> Result<()> {
    println!("Starting");

    let device = Device::cuda_if_available(0)?;
    let vm = VarMap::new();
    let vb = VarBuilder::from_varmap(&vm, DType::F32, &device.clone());
    let patch_zero: PatchZero = PatchZero::new(vb, device)?;

    let states = vec![
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
        patchwork_core::Patchwork::get_initial_state(None),
    ];

    let now = std::time::Instant::now();
    let (policies, values) = patch_zero.forward_t(&states.iter().collect::<Vec<_>>(), false)?;

    let _first_policy = policies.i((0, ..))?;
    let _first_value = values.i((0, ..))?.squeeze(0)?.to_scalar::<f32>()?;

    println!("Forward pass done in {:?}", now.elapsed());
    println!("Policies: {:?}", policies);
    println!("Values: {:?}", values);
    Ok(())
}
