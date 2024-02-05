use alphazero_player::PatchZero;
use candle_core::{DType, Device, Result};
use candle_nn::{VarBuilder, VarMap};

fn main() -> Result<()> {
    println!("Starting");

    let device = &Device::Cpu;
    let vm = VarMap::new();
    let vb = VarBuilder::from_varmap(&vm, DType::F32, device);
    let patch_zero: PatchZero = PatchZero::new(vb, device)?;
    let state = patchwork_core::Patchwork::get_initial_state(None);

    let now = std::time::Instant::now();
    let (policy, value) = patch_zero.forward_t(&state, false)?;
    println!("Forward pass done in {:?}", now.elapsed());
    println!("Policy: {:?}", policy);
    println!("Value: {:?}", value);
    Ok(())
}
