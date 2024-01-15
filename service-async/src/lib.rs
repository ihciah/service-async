use std::future::Future;

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
pub use boxed::{BoxService, BoxServiceFactory, BoxedAsyncMakeService, BoxedService};
mod make_service;
pub use make_service::{
    ArcMakeBoxedService, ArcMakeService, AsyncMakeService, AsyncMakeServiceWrapper,
    BoxedMakeBoxedService, BoxedMakeService, MakeService,
};

pub trait Service<Request> {
    /// Responses given by the service.
    type Response;
    /// Errors produced by the service.
    type Error;

    /// Process the request and return the response asynchronously.
    fn call(&self, req: Request) -> impl Future<Output = Result<Self::Response, Self::Error>>;
}
