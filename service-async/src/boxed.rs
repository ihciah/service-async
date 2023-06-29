use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

pub use futures_util::future::LocalBoxFuture;

use crate::{MakeService, Service};

pub struct BoxedService<Request, Response, E> {
    svc: *const (),
    type_id: TypeId,
    vtable: ServiceVtable<Request, Response, E>,
}

impl<Request, Response, E> BoxedService<Request, Response, E> {
    pub fn new<S>(s: S) -> Self
    where
        S: Service<Request, Response = Response, Error = E> + 'static,
        Request: 'static,
    {
        let type_id = s.type_id();
        let svc = Box::into_raw(Box::new(s)) as *const ();
        BoxedService {
            svc,
            type_id,
            vtable: ServiceVtable {
                call: call::<Request, S>,
                drop: drop::<S>,
            },
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        let t = TypeId::of::<T>();
        if self.type_id == t {
            Some(unsafe { self.downcast_ref_unchecked() })
        } else {
            None
        }
    }

    /// # Safety
    /// If you are sure the inner type is T, you can downcast it.
    pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
        &*(self.svc as *const () as *const T)
    }
}

impl<Request, Response, E> Drop for BoxedService<Request, Response, E> {
    #[inline]
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.svc) };
    }
}

impl<Request, Response, E> Service<Request> for BoxedService<Request, Response, E> {
    type Response = Response;
    type Error = E;
    type Future<'cx> = LocalBoxFuture<'cx, Result<Response, E>>
    where
        Self: 'cx, Request: 'cx;

    #[inline]
    fn call(&self, req: Request) -> Self::Future<'_> {
        unsafe { (self.vtable.call)(self.svc, req) }
    }
}

pub trait BoxService<Request, Response, E> {
    fn into_boxed(self) -> BoxedService<Request, Response, E>;
}

impl<T, Request, Response, E> BoxService<Request, Response, E> for T
where
    T: Service<Request, Response = Response, Error = E> + 'static,
    Request: 'static,
{
    fn into_boxed(self) -> BoxedService<Request, Response, E> {
        BoxedService::new(self)
    }
}

struct ServiceVtable<T, U, E> {
    call: unsafe fn(raw: *const (), req: T) -> LocalBoxFuture<'static, Result<U, E>>,
    drop: unsafe fn(raw: *const ()),
}

unsafe fn call<R, S>(
    svc: *const (),
    req: R,
) -> LocalBoxFuture<'static, Result<S::Response, S::Error>>
where
    R: 'static,
    S: Service<R> + 'static,
{
    let svc = &*svc.cast::<S>();
    let fut = S::call(svc, req);
    Box::pin(fut)
}

unsafe fn drop<S>(raw: *const ()) {
    std::ptr::drop_in_place(raw as *mut S);
}

pub struct BoxServiceFactory<F, Req>
where
    F: MakeService,
    F::Service: Service<Req>,
{
    pub inner: F,
    _marker: PhantomData<Req>,
}

impl<F, Req> BoxServiceFactory<F, Req>
where
    F: MakeService,
    F::Service: Service<Req>,
{
    pub fn new(inner: F) -> Self {
        BoxServiceFactory {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<F, Req> MakeService for BoxServiceFactory<F, Req>
where
    F: MakeService,
    F::Service: Service<Req> + 'static,
    Req: 'static,
{
    type Service = BoxedService<
        Req,
        <F::Service as Service<Req>>::Response,
        <F::Service as Service<Req>>::Error,
    >;
    type Error = F::Error;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        let svc = match old {
            Some(inner) => self.inner.make_via_ref(inner.downcast_ref())?,
            None => self.inner.make()?,
        };
        Ok(svc.into_boxed())
    }
}
