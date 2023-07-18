#![feature(impl_trait_in_assoc_type)]

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

    /// The future response value.
    type Future<'cx>: Future<Output = Result<Self::Response, Self::Error>>
    where
        Self: 'cx,
        Request: 'cx;

    /// Process the request and return the response asynchronously.
    fn call(&self, req: Request) -> Self::Future<'_>;
}

pub trait MakeService {
    type Service;
    type Error;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error>;
    fn make(&self) -> Result<Self::Service, Self::Error> {
        self.make_via_ref(None)
    }
}

impl<T: MakeService> MakeService for &T {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        (*self).make_via_ref(old)
    }
}

impl<T: MakeService> MakeService for Arc<T> {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old)
    }
}

pub type BoxedMakeService<S, E> = Box<dyn MakeService<Service = S, Error = E>>;
pub type BoxedMakeBoxedService<Req, Resp, SE, ME> =
    BoxedMakeService<BoxedService<Req, Resp, SE>, ME>;
