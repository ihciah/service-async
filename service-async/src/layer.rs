use std::marker::PhantomData;

use crate::AsyncMakeServiceWrapper;

pub trait FactoryLayer<C, F> {
    type Factory;
    fn layer(&self, config: &C, inner: F) -> Self::Factory;
}

pub const fn layer_fn<C, FN>(f: FN) -> LayerFn<C, FN> {
    LayerFn {
        f,
        marker: PhantomData,
    }
}

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
