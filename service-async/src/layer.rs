use std::marker::PhantomData;

use crate::AsyncMakeServiceWrapper;

/// A trait for creating layered factory wrappers, enabling complex service compositions.
///
/// `FactoryLayer` defines how to wrap one factory with another, creating a new composite factory.
/// This allows for the creation of reusable, modular pieces of functionality that can be easily combined.
///
/// Unlike Tower's `Layer` which creates a `Service` wrapping an inner `Service`,
/// `FactoryLayer` creates a `Factory` wrapping an inner `Factory`, which can then be used
/// to create the entire `Service` chain.
pub trait FactoryLayer<C, F> {
    /// The type of factory this layer produces.
    type Factory;

    /// Creates a new factory wrapper.
    ///
    /// This method defines how the layer transforms the inner factory into a new factory.
    fn layer(&self, config: &C, inner: F) -> Self::Factory;
}

/// Creates a `FactoryLayer` from a closure, simplifying the creation of custom layers.
///
/// This function allows for easy creation of `FactoryLayer` implementations without
/// explicitly defining new structs.
pub const fn layer_fn<C, FN>(f: FN) -> LayerFn<C, FN> {
    LayerFn {
        f,
        marker: PhantomData,
    }
}

/// A struct that wraps a closure to implement `FactoryLayer`.
///
/// `LayerFn` allows closures to be used as `FactoryLayer`s, providing a flexible way
/// to create custom layers.
pub struct LayerFn<C, FN> {
    f: FN,
    marker: PhantomData<fn(C)>,
}

impl<C, F, FN, O> FactoryLayer<C, F> for LayerFn<C, FN>
where
    FN: Fn(&C, F) -> O,
{
    type Factory = O;

    #[inline]
    fn layer(&self, config: &C, inner: F) -> Self::Factory {
        (self.f)(config, inner)
    }
}

pub struct LayerAsync;

impl<C, F> FactoryLayer<C, F> for LayerAsync {
    type Factory = AsyncMakeServiceWrapper<F>;

    #[inline]
    fn layer(&self, _config: &C, inner: F) -> Self::Factory {
        AsyncMakeServiceWrapper(inner)
    }
}
