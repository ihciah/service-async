use crate::{boxed::BoxServiceFactory, BoxedMakeService, BoxedService};

use super::{layer::FactoryLayer, MakeService, MapTargetService, Service};

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

    /// Push a new factory of BoxedService.
    #[inline]
    pub fn push_boxed_service<Req>(self) -> FactoryStack<C, BoxServiceFactory<F, Req>>
    where
        F: MakeService,
        F::Service: Service<Req>,
    {
        FactoryStack {
            config: self.config,
            inner: BoxServiceFactory::new(self.inner),
        }
    }

    /// Push a new factory wrapper to get a fixed type factory.
    #[inline]
    pub fn push_boxed_factory<Req, Resp, SE, ME>(
        self,
    ) -> FactoryStack<C, BoxedMakeService<Req, Resp, SE, ME>>
    where
        F: MakeService<Service = BoxedService<Req, Resp, SE>, Error = ME> + 'static,
    {
        FactoryStack {
            config: self.config,
            inner: Box::new(self.inner),
        }
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
