use std::{future::Future, sync::Arc};

pub trait MakeService {
    type Service;
    type Error;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error>;
    fn make(&self) -> Result<Self::Service, Self::Error> {
        self.make_via_ref(None)
    }
}

impl<T: MakeService + ?Sized> MakeService for &T {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        (*self).make_via_ref(old)
    }
}

impl<T: MakeService + ?Sized> MakeService for Arc<T> {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old)
    }
}

impl<T: MakeService + ?Sized> MakeService for Box<T> {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old)
    }
}

pub type BoxedMakeService<S, E> =
    Box<dyn MakeService<Service = S, Error = E> + Send + Sync + 'static>;
pub type ArcMakeService<S, E> =
    Arc<dyn MakeService<Service = S, Error = E> + Send + Sync + 'static>;
pub type BoxedMakeBoxedService<Req, Resp, SE, ME> =
    BoxedMakeService<crate::BoxedService<Req, Resp, SE>, ME>;
pub type ArcMakeBoxedService<Req, Resp, SE, ME> =
    ArcMakeService<crate::BoxedService<Req, Resp, SE>, ME>;

pub trait AsyncMakeService {
    type Service;
    type Error;

    fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>>;
    fn make(&self) -> impl Future<Output = Result<Self::Service, Self::Error>> {
        self.make_via_ref(None)
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for &T {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        (*self).make_via_ref(old).await
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for Arc<T> {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old).await
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for Box<T> {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old).await
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncMakeServiceWrapper<T>(pub T);

impl<T: MakeService> AsyncMakeService for AsyncMakeServiceWrapper<T> {
    type Service = <T as MakeService>::Service;
    type Error = <T as MakeService>::Error;

    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        <T as MakeService>::make_via_ref(&self.0, old)
    }
    async fn make(&self) -> Result<Self::Service, Self::Error> {
        <T as MakeService>::make(&self.0)
    }
}
