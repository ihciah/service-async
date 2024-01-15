use std::sync::Arc;

use crate::{AsyncMakeServiceWrapper, BoxedAsyncMakeService};

use super::{
    boxed::BoxServiceFactory, layer::FactoryLayer, ArcMakeService, AsyncMakeService,
    BoxedMakeService, MakeService, MapTargetService, Service,
};

pub struct FactoryStack<C, S> {
    config: C,
    inner: S,
}

impl<C> FactoryStack<C, ()> {
    pub const fn new(config: C) -> Self {
        FactoryStack { config, inner: () }
    }
}

impl<C, F> FactoryStack<C, F> {
    /// Replace inner with a new factory.
    #[inline]
    pub fn replace<NF>(self, factory: NF) -> FactoryStack<C, NF> {
        FactoryStack {
            config: self.config,
            inner: factory,
        }
    }

    /// Push a new factory layer.
    #[inline]
    pub fn push<L>(self, layer: L) -> FactoryStack<C, L::Factory>
    where
        L: FactoryLayer<C, F>,
    {
        let inner = layer.layer(&self.config, self.inner);
        FactoryStack {
            config: self.config,
            inner,
        }
    }

    /// Convert the factory to an async factory.
    #[inline]
    pub fn into_async(self) -> FactoryStack<C, AsyncMakeServiceWrapper<F>> {
        let inner = AsyncMakeServiceWrapper(self.inner);
        FactoryStack {
            config: self.config,
            inner,
        }
    }

    /// Push a new factory of service to map the request type.
    #[inline]
    pub fn push_map_target<M: Clone>(self, f: M) -> FactoryStack<C, MapTargetService<F, M>> {
        FactoryStack {
            config: self.config,
            inner: MapTargetService {
                f,
                inner: self.inner,
            },
        }
    }

    /// Convert the factory to factory of BoxedService.
    /// Works for MakeService and AsyncMakeService.
    #[inline]
    pub fn into_boxed_service<Req>(self) -> FactoryStack<C, BoxServiceFactory<F, Req>> {
        FactoryStack {
            config: self.config,
            inner: BoxServiceFactory::new(self.inner),
        }
    }

    /// Convert the factory to factory of BoxedService.
    /// Works for MakeService and AsyncMakeService.
    #[deprecated = "use `into_boxed_service` instead"]
    #[inline]
    pub fn push_boxed_service<Req>(self) -> FactoryStack<C, BoxServiceFactory<F, Req>>
    where
        F: MakeService,
        F::Service: Service<Req>,
    {
        self.into_boxed_service()
    }

    /// Convert the factory to a fixed type factory(Box dyn).
    /// Only works for MakeService.
    #[inline]
    pub fn into_boxed_factory(self) -> FactoryStack<C, BoxedMakeService<F::Service, F::Error>>
    where
        F: MakeService + Send + Sync + 'static,
    {
        FactoryStack {
            config: self.config,
            inner: Box::new(self.inner),
        }
    }

    /// Convert the factory to a fixed type factory(Box dyn).
    /// Only works for AsyncMakeService.
    #[inline]
    pub fn into_async_boxed_factory(
        self,
    ) -> FactoryStack<C, BoxedAsyncMakeService<F::Service, F::Error>>
    where
        F: AsyncMakeService + 'static,
        F::Service: 'static,
    {
        FactoryStack {
            config: self.config,
            inner: BoxedAsyncMakeService::new(self.inner),
        }
    }

    /// Convert the factory to a fixed type factory(Box dyn).
    /// Only works for MakeService.
    #[deprecated = "use `into_boxed_factory` instead"]
    #[inline]
    pub fn push_boxed_factory(self) -> FactoryStack<C, BoxedMakeService<F::Service, F::Error>>
    where
        F: MakeService + Send + Sync + 'static,
    {
        self.into_boxed_factory()
    }

    /// Convert the factory to a fixed type factory(Arc dyn).
    /// Only works for MakeService.
    #[inline]
    pub fn into_arc_factory(self) -> FactoryStack<C, ArcMakeService<F::Service, F::Error>>
    where
        F: MakeService + Send + Sync + 'static,
    {
        FactoryStack {
            config: self.config,
            inner: Arc::new(self.inner),
        }
    }

    /// Convert the factory to a fixed type factory(Arc Box dyn).
    /// Only works for AsyncMakeService.
    #[allow(clippy::type_complexity)]
    #[inline]
    pub fn into_async_arc_factory(
        self,
    ) -> FactoryStack<C, Arc<BoxedAsyncMakeService<F::Service, F::Error>>>
    where
        F: AsyncMakeService + 'static,
        F::Service: 'static,
    {
        FactoryStack {
            config: self.config,
            inner: Arc::new(BoxedAsyncMakeService::new(self.inner)),
        }
    }

    /// Convert the factory to a fixed type factory(Arc dyn).
    /// Only works for MakeService.
    #[deprecated = "use `into_arc_factory` instead"]
    #[inline]
    pub fn push_arc_factory(self) -> FactoryStack<C, ArcMakeService<F::Service, F::Error>>
    where
        F: MakeService + Send + Sync + 'static,
    {
        self.into_arc_factory()
    }

    /// Check if the stack is a factory of Service<R>.
    #[inline]
    pub fn check_make_svc<R>(self) -> Self
    where
        F: MakeService,
        F::Service: Service<R>,
    {
        self
    }

    /// Check if the stack is an async factory of Service<R>.
    #[inline]
    pub fn check_async_make_svc<R>(self) -> Self
    where
        F: AsyncMakeService,
        F::Service: Service<R>,
    {
        self
    }

    /// Get the inner factory.
    #[inline]
    pub fn into_inner(self) -> F {
        self.inner
    }

    /// Into config and the factory.
    #[inline]
    pub fn into_parts(self) -> (C, F) {
        (self.config, self.inner)
    }
}

impl<C, F> FactoryStack<C, F>
where
    F: MakeService,
{
    /// Make a service.
    #[inline]
    pub fn make(&self) -> Result<F::Service, F::Error> {
        self.inner.make()
    }
}

impl<C, F> FactoryStack<C, F>
where
    F: AsyncMakeService,
{
    /// Make a service in async.
    #[inline]
    pub async fn make_async(&self) -> Result<F::Service, F::Error> {
        self.inner.make().await
    }
}
