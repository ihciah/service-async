use std::future::Future;

use super::{AsyncMakeService, MakeService, Service};

pub trait MapTarget<T> {
    type Target;

    fn map_target(&self, t: T) -> Self::Target;
}

impl<F, T, U> MapTarget<T> for F
where
    F: Fn(T) -> U,
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

impl<T, F, R> Service<R> for MapTargetService<T, F>
where
    F: MapTarget<R>,
    T: Service<F::Target>,
{
    type Response = T::Response;

    type Error = T::Error;

    #[inline]
    fn call(&self, req: R) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        let req = self.f.map_target(req);
        self.inner.call(req)
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

impl<FAC, F> AsyncMakeService for MapTargetService<FAC, F>
where
    FAC: AsyncMakeService,
    F: Clone,
{
    type Service = MapTargetService<FAC::Service, F>;
    type Error = FAC::Error;

    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        Ok(MapTargetService {
            f: self.f.clone(),
            inner: self
                .inner
                .make_via_ref(old.map(|o| &o.inner))
                .await
                .map_err(Into::into)?,
        })
    }
}
