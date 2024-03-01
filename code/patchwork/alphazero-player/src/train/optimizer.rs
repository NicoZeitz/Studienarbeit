// Code adapted from [optim.rs](https://github.com/huggingface/candle/blob/main/candle-nn/src/optim.rs)

use std::collections::HashMap;

use candle_core::{backprop::GradStore, safetensors, Device, IndexOp, Result, Tensor, Var};
use candle_nn::Optimizer;

#[derive(Clone, Debug)]
pub struct ParamsAdamW {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub eps: f64,
    pub weight_decay: f64,
}

impl Default for ParamsAdamW {
    fn default() -> Self {
        Self {
            lr: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.01,
        }
    }
}

#[derive(Debug)]
struct VarAdamW {
    var: Var,
    first_moment: Var,
    second_moment: Var,
}

#[derive(Debug)]
pub struct AdamW {
    vars: Vec<VarAdamW>,
    step_t: usize,
    params: ParamsAdamW,
}

impl Optimizer for AdamW {
    type Config = ParamsAdamW;

    fn new(vars: Vec<Var>, params: ParamsAdamW) -> Result<Self> {
        let vars = vars
            .into_iter()
            .filter(|var| var.dtype().is_float())
            .map(|var| {
                let dtype = var.dtype();
                let shape = var.shape();
                let device = var.device();
                let first_moment = Var::zeros(shape, dtype, device)?;
                let second_moment = Var::zeros(shape, dtype, device)?;
                Ok(VarAdamW {
                    var,
                    first_moment,
                    second_moment,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            vars,
            params,
            step_t: 0,
        })
    }

    fn learning_rate(&self) -> f64 {
        self.params.lr
    }

    fn set_learning_rate(&mut self, lr: f64) {
        self.params.lr = lr;
    }

    fn step(&mut self, grads: &GradStore) -> Result<()> {
        self.step_t += 1;
        let lr = self.params.lr;
        let lambda = self.params.weight_decay;
        let lr_lambda = lr * lambda;
        let beta1 = self.params.beta1;
        let beta2 = self.params.beta2;
        let scale_m = 1f64 / (1f64 - beta1.powi(self.step_t as i32));
        let scale_v = 1f64 / (1f64 - beta2.powi(self.step_t as i32));
        for var in &self.vars {
            let theta = &var.var;
            let m = &var.first_moment;
            let v = &var.second_moment;
            if let Some(g) = grads.get(theta) {
                // This involves locking 3 RWLocks per params, if the parameters are large this
                // should not be an issue but this may be problematic with models with lots of
                // small parameters.
                let next_m = ((m.as_tensor() * beta1)? + (g * (1.0 - beta1))?)?;
                let next_v = ((v.as_tensor() * beta2)? + (g.sqr()? * (1.0 - beta2))?)?;
                let m_hat = (&next_m * scale_m)?;
                let v_hat = (&next_v * scale_v)?;
                let next_theta = (theta.as_tensor() * (1f64 - lr_lambda))?;
                let adjusted_grad = (m_hat / (v_hat.sqrt()? + self.params.eps)?)?;
                let next_theta = (next_theta - (adjusted_grad * lr)?)?;
                m.set(&next_m)?;
                v.set(&next_v)?;
                theta.set(&next_theta)?;
            }
        }
        Ok(())
    }
}

impl AdamW {
    /// Save the optimizers state in the safetensors format.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the state to.
    ///
    /// # Errors
    ///
    /// If there is an error saving the state to the file.
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut data = HashMap::<String, Tensor>::new();
        data.insert(
            "step_t".to_string(),
            Tensor::from_slice(&[self.step_t as f64], (1,), &Device::Cpu)?,
        );
        data.insert(
            "params.lr".to_string(),
            Tensor::from_slice(&[self.params.lr], (1,), &Device::Cpu)?,
        );
        data.insert(
            "params.beta1".to_string(),
            Tensor::from_slice(&[self.params.beta1], (1,), &Device::Cpu)?,
        );
        data.insert(
            "params.beta2".to_string(),
            Tensor::from_slice(&[self.params.beta2], (1,), &Device::Cpu)?,
        );
        data.insert(
            "params.eps".to_string(),
            Tensor::from_slice(&[self.params.eps], (1,), &Device::Cpu)?,
        );
        data.insert(
            "params.weight_decay".to_string(),
            Tensor::from_slice(&[self.params.weight_decay], (1,), &Device::Cpu)?,
        );
        data.insert(
            "vars".to_string(),
            Tensor::stack(
                self.vars
                    .iter()
                    .flat_map(|var| {
                        vec![
                            var.var.as_tensor(),
                            var.first_moment.as_tensor(),
                            var.second_moment.as_tensor(),
                        ]
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
                0,
            )?,
        );

        safetensors::save(&data, path.as_ref())
    }

    /// Load some values from a safetensors file and modify the existing state to have these
    /// values.
    ///
    /// Note that values for variables that are currently not in the map are not kept.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load the state from.
    ///
    /// # Errors
    ///
    /// If there is an error loading the state from the file.
    ///
    /// # Panics
    ///
    /// If the state is not in the expected format.
    pub fn load<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let data = safetensors::load(path.as_ref(), &Device::Cpu)?;
        self.step_t = data.get("step_t").unwrap().to_scalar::<f64>()? as usize;
        self.params.lr = data.get("params.lr").unwrap().to_scalar::<f64>()?;
        self.params.beta1 = data.get("params.beta1").unwrap().to_scalar::<f64>()?;
        self.params.beta2 = data.get("params.beta2").unwrap().to_scalar::<f64>()?;
        self.params.eps = data.get("params.eps").unwrap().to_scalar::<f64>()?;
        self.params.weight_decay = data.get("params.weight_decay").unwrap().to_scalar::<f64>()?;

        let vars = data.get("vars").unwrap();
        let mut dim_0 = 0;
        while dim_0 < vars.dims()[0] {
            let var = &vars.i((dim_0, ..))?;
            let first_moment = &vars.i((dim_0 + 1, ..))?;
            let second_moment = &vars.i((dim_0 + 2, ..))?;
            self.vars[dim_0 / 3].var.set(var)?;
            self.vars[dim_0 / 3].first_moment.set(first_moment)?;
            self.vars[dim_0 / 3].second_moment.set(second_moment)?;
            dim_0 += 3;
        }

        Ok(())
    }

    #[must_use]
    pub const fn params(&self) -> &ParamsAdamW {
        &self.params
    }

    pub fn set_params(&mut self, params: ParamsAdamW) {
        self.params = params;
    }
}
