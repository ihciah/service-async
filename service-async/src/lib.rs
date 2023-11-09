use std::{future::Future, sync::Arc};

pub mod either;
pub mod layer;
pub mod stack;
pub mod utils;

pub use param::{
    Param, ParamMaybeMut, ParamMaybeRef, ParamMut, ParamRef, ParamRemove, ParamSet, ParamTake,
};
mod map;
pub use map::MapTargetService;
mod boxed;
pub use boxed::{BoxService, BoxServiceFactory, BoxedService};

pub trait Service<Request> {
    /// Responses given by the service.
    type Response;
    /// Errors produced by the service.
    type Error;

    /// Process the request and return the response asynchronously.
    fn call(&self, req: Request) -> impl Future<Output = Result<Self::Response, Self::Error>>;
}

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
    BoxedMakeService<BoxedService<Req, Resp, SE>, ME>;
pub type ArcMakeBoxedService<Req, Resp, SE, ME> = ArcMakeService<BoxedService<Req, Resp, SE>, ME>;
