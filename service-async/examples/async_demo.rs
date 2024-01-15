use std::{any::Any, convert::Infallible};

#[cfg(unix)]
use monoio::main as main_macro;
use service_async::{
    layer::{layer_fn, FactoryLayer},
    stack::FactoryStack,
    AsyncMakeService, BoxedService, MakeService, Service,
};
#[cfg(not(unix))]
use tokio::main as main_macro;

struct SvcA;

impl Service<()> for SvcA {
    type Response = ();
    type Error = Infallible;
    async fn call(&self, _req: ()) -> Result<Self::Response, Self::Error> {
        println!("SvcA called!");
        Ok(())
    }
}

struct SvcAFactory;

impl MakeService for SvcAFactory {
    type Service = SvcA;
    type Error = Infallible;
    fn make_via_ref(&self, _old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        println!("SvcAFactory make");
        Ok(SvcA)
    }
}
impl AsyncMakeService for SvcAFactory {
    type Service = SvcA;
    type Error = Infallible;
    async fn make_via_ref(
        &self,
        _old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        println!("SvcAFactory make async");
        Ok(SvcA)
    }
}

struct SvcB<T> {
    inner: T,
}

impl<T: Service<()>> Service<()> for SvcB<T> {
    type Response = ();
    type Error = T::Error;
    async fn call(&self, req: ()) -> Result<Self::Response, Self::Error> {
        println!("SvcB called!");
        self.inner.call(req).await?;
        Ok(())
    }
}

struct SvcBFactory<T>(T);

impl<T: AsyncMakeService> AsyncMakeService for SvcBFactory<T> {
    type Service = SvcB<T::Service>;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        _old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        println!("SvcBFactory make async");
        Ok(SvcB {
            inner: self.0.make_via_ref(None).await?,
        })
    }
}

impl SvcAFactory {
    fn layer<C>() -> impl FactoryLayer<C, (), Factory = Self> {
        layer_fn(|_c: &C, ()| SvcAFactory)
    }
}
impl<T> SvcBFactory<T> {
    fn layer<C>() -> impl FactoryLayer<C, T, Factory = Self> {
        layer_fn(|_: &C, inner| SvcBFactory(inner))
    }
}

#[main_macro]
async fn main() {
    // Demo for normal async make service.
    let stack = FactoryStack::new(())
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer());
    let svc = stack.make_async().await.unwrap();
    svc.call(()).await.unwrap();

    // Demo for convert make service to async make service.
    let stack = FactoryStack::new(())
        .push(SvcAFactory::layer())
        .check_make_svc()
        .into_async()
        .check_async_make_svc()
        .push(SvcBFactory::layer());
    let svc = stack.make_async().await.unwrap();
    svc.call(()).await.unwrap();

    // Demo for convert service type to BoxedService and factory to BoxedAsyncMakeService<S, E>.
    let stack = FactoryStack::new(())
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer())
        .into_boxed_service()
        .into_async_boxed_factory();
    let svc = stack.make_async().await.unwrap();
    assert!(svc.type_id() == std::any::TypeId::of::<BoxedService<(), (), Infallible>>());
    svc.call(()).await.unwrap();
}
