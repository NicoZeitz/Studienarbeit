//! A sequential layer used to chain multiple layers and closures.
//! Adapted from candle_nn but with the following changes:
//! - Use the ModuleT trait instead of Module to allow for training
use candle_core::{ModuleT, Result, Tensor};

/// A sequential layer combining multiple other layers.
pub struct Sequential {
    layers: Vec<Box<dyn ModuleT>>,
}

/// Creates a new empty sequential layer.
pub fn seq() -> Sequential {
    Sequential { layers: vec![] }
}

impl Sequential {
    /// The number of sub-layers embedded in this layer.
    #[allow(dead_code)]
    pub fn len(&self) -> i64 {
        self.layers.len() as i64
    }

    /// Returns true if this layer does not have any sub-layer.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }
}

impl ModuleT for Sequential {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let mut xs = xs.clone();
        for layer in self.layers.iter() {
            xs = layer.forward_t(&xs, train)?
        }
        Ok(xs)
    }
}

impl Sequential {
    /// Appends a layer after all the current layers.
    #[allow(clippy::should_implement_trait)]
    pub fn add<M: ModuleT + 'static>(mut self, layer: M) -> Self {
        self.layers.push(Box::new(layer));
        self
    }

    /// Appends a closure after all the current layers.
    pub fn add_fn<F>(self, f: F) -> Self
    where
        F: 'static + Fn(&Tensor) -> Result<Tensor> + Send + Sync,
    {
        self.add(candle_nn::func_t(move |args, _| f(args)))
    }

    /// Appends a closure after all the current layers.
    #[allow(dead_code)]
    pub fn add_fn_t<F>(self, f: F) -> Self
    where
        F: 'static + Fn(&Tensor, bool) -> Result<Tensor> + Send + Sync,
    {
        self.add(candle_nn::func_t(f))
    }

    /// Applies the forward pass and returns the output for each layer.
    #[allow(dead_code)]
    pub fn forward_all_t(&self, xs: &Tensor, train: bool) -> Result<Vec<Tensor>> {
        let mut vec = Vec::with_capacity(self.layers.len());
        let mut xs = xs.clone();
        for layer in self.layers.iter() {
            xs = layer.forward_t(&xs, train)?;
            vec.push(xs.clone())
        }
        Ok(vec)
    }
}
