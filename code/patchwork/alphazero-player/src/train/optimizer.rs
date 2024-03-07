// Code adapted from [optim.rs](https://github.com/huggingface/candle/blob/main/candle-nn/src/optim.rs)

use std::collections::HashMap;

use candle_core::{backprop::GradStore, safetensors, Device, Result, Tensor, Var};
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

#[derive(Debug, Clone)]
struct VarAdamW {
    var: Var,
    first_moment: Var,
    second_moment: Var,
}

#[derive(Debug, Clone)]
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
        data.insert("step_t".to_string(), Tensor::new(self.step_t as f64, &Device::Cpu)?);
        data.insert("params.lr".to_string(), Tensor::new(self.params.lr, &Device::Cpu)?);
        data.insert(
            "params.beta1".to_string(),
            Tensor::new(self.params.beta1, &Device::Cpu)?,
        );
        data.insert(
            "params.beta2".to_string(),
            Tensor::new(self.params.beta2, &Device::Cpu)?,
        );
        data.insert("params.eps".to_string(), Tensor::new(self.params.eps, &Device::Cpu)?);
        data.insert(
            "params.weight_decay".to_string(),
            Tensor::new(self.params.weight_decay, &Device::Cpu)?,
        );
        for (index, var) in self.vars.iter().enumerate() {
            data.insert(format!("vars.{index}.var"), var.var.as_detached_tensor());
            data.insert(
                format!("vars.{index}.first_moment"),
                var.first_moment.as_detached_tensor(),
            );
            data.insert(
                format!("vars.{index}.second_moment"),
                var.second_moment.as_detached_tensor(),
            );
        }

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

        for (index, var_adamw) in self.vars.iter_mut().enumerate() {
            let var = data.get(&format!("vars.{index}.var")).unwrap();
            let first_moment = data.get(&format!("vars.{index}.first_moment")).unwrap();
            let second_moment = data.get(&format!("vars.{index}.second_moment")).unwrap();

            var_adamw.var.set(var)?;
            var_adamw.first_moment.set(first_moment)?;
            var_adamw.second_moment.set(second_moment)?;
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
