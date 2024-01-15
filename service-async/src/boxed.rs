use std::{
    any::{Any, TypeId},
    future::Future,
    marker::PhantomData,
    pin::Pin,
};

use crate::{AsyncMakeService, MakeService, Service};

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
        &*(self.svc as *const T)
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

    #[inline]
    fn call(&self, req: Request) -> impl Future<Output = Result<Self::Response, Self::Error>> {
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

type LocalStaticBoxedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>;

struct ServiceVtable<T, U, E> {
    call: unsafe fn(raw: *const (), req: T) -> LocalStaticBoxedFuture<U, E>,
    drop: unsafe fn(raw: *const ()),
}

unsafe fn call<R, S>(svc: *const (), req: R) -> LocalStaticBoxedFuture<S::Response, S::Error>
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

pub struct BoxServiceFactory<F, Req> {
    pub inner: F,
    _marker: PhantomData<Req>,
}

unsafe impl<F: Send, Req> Send for BoxServiceFactory<F, Req> {}

unsafe impl<F: Sync, Req> Sync for BoxServiceFactory<F, Req> {}

impl<F, Req> BoxServiceFactory<F, Req> {
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

impl<F, Req> AsyncMakeService for BoxServiceFactory<F, Req>
where
    F: AsyncMakeService,
    F::Service: Service<Req> + 'static,
    Req: 'static,
{
    type Service = BoxedService<
        Req,
        <F::Service as Service<Req>>::Response,
        <F::Service as Service<Req>>::Error,
    >;
    type Error = F::Error;

    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        let svc = match old {
            Some(inner) => self.inner.make_via_ref(inner.downcast_ref()).await?,
            None => self.inner.make().await?,
        };
        Ok(svc.into_boxed())
    }
}

pub struct BoxedAsyncMakeService<S, E> {
    svc: *const (),
    type_id: TypeId,
    vtable: AsyncMakeServiceVtable<S, E>,
}

impl<S, E> BoxedAsyncMakeService<S, E> {
    pub fn new<AMS>(ams: AMS) -> Self
    where
        AMS: AsyncMakeService<Service = S, Error = E> + 'static,
        S: 'static,
    {
        let type_id = ams.type_id();
        let svc = Box::into_raw(Box::new(ams)) as *const ();
        BoxedAsyncMakeService {
            svc,
            type_id,
            vtable: AsyncMakeServiceVtable {
                make_via_ref: make_via_ref::<AMS, S, E>,
                drop: drop::<AMS>,
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
        &*(self.svc as *const T)
    }
}

impl<S, E> Drop for BoxedAsyncMakeService<S, E> {
    #[inline]
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.svc) };
    }
}

impl<S, E> AsyncMakeService for BoxedAsyncMakeService<S, E> {
    type Service = S;
    type Error = E;

    #[inline]
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        unsafe { (self.vtable.make_via_ref)(self.svc, old.map(|s| s as _)) }.await
    }
}

type LocalBoxedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

struct AsyncMakeServiceVtable<S, E> {
    make_via_ref: unsafe fn(raw: *const (), old: Option<*const S>) -> LocalBoxedFuture<S, E>,
    drop: unsafe fn(raw: *const ()),
}

unsafe fn make_via_ref<AMS, S, E>(
    svc: *const (),
    old: Option<*const AMS::Service>,
) -> LocalBoxedFuture<S, E>
where
    AMS: AsyncMakeService<Service = S, Error = E> + 'static,
    S: 'static,
{
    let svc = &*svc.cast::<AMS>();
    let fut = AMS::make_via_ref(svc, old.map(|s| unsafe { &*s }));
    Box::pin(fut)
}
