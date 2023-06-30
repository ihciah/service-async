use std::future::Future;

use async_trait::async_trait;

use super::{MakeService, Service};

pub trait MapTarget<T>: Send + Sync {
    type Target;

    fn map_target(&self, t: T) -> Self::Target;
}

impl<F, T, U> MapTarget<T> for F
where
    F: Fn(T) -> U + Send + Sync,
{
    type Target = U;

    #[inline]
    fn map_target(&self, t: T) -> U {
        (self)(t)
    }
}

pub struct MapTargetService<T, F> {
    pub f: F,
    pub inner: T,
}

#[async_trait]
impl<T, F, R> Service<R> for MapTargetService<T, F>
where
    F: MapTarget<R>,
    T: Service<F::Target>,
    R: Send + 'static,
    F::Target: Send,
{
    type Response = T::Response;

    type Error = T::Error;

    #[inline]
    async fn call(&self, req: R) -> Result<Self::Response, Self::Error> {
        let req = self.f.map_target(req);
        self.inner.call(req).await
    }
}

impl<FAC, F> MakeService for MapTargetService<FAC, F>
where
    FAC: MakeService,
    F: Clone,
{
    type Service = MapTargetService<FAC::Service, F>;
    type Error = FAC::Error;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(MapTargetService {
            f: self.f.clone(),
            inner: self
                .inner
                .make_via_ref(old.map(|o| &o.inner))
                .map_err(Into::into)?,
        })
    }
}
