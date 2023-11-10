use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

use service_async::{
    layer::{layer_fn, FactoryLayer},
    stack::FactoryStack,
    BoxedMakeService, BoxedService, MakeService, Param, Service,
};

#[cfg(unix)]
use monoio::main as main_macro;
#[cfg(not(unix))]
use tokio::main as main_macro;

// ===== Svc*(impl Service) and Svc*Factory(impl NewService) =====

struct SvcA {
    pass_flag: bool,
    not_pass_flag: bool,
}

impl Service<()> for SvcA {
    type Response = ();
    type Error = Infallible;

    async fn call(&self, _req: ()) -> Result<Self::Response, Self::Error> {
        println!(
            "SvcA called! pass_flag = {}, not_pass_flag = {}",
            self.pass_flag, self.not_pass_flag
        );
        Ok(())
    }
}

struct SvcAFactory {
    init_flag: InitFlag,
}

struct InitFlag(bool);

impl MakeService for SvcAFactory {
    type Service = SvcA;
    type Error = Infallible;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(match old {
            Some(r) => SvcA {
                pass_flag: r.pass_flag,
                not_pass_flag: self.init_flag.0,
            },
            None => SvcA {
                pass_flag: self.init_flag.0,
                not_pass_flag: self.init_flag.0,
            },
        })
    }
}

struct SvcB<T> {
    counter: AtomicUsize,
    inner: T,
}

impl<T> Service<usize> for SvcB<T>
where
    T: Service<(), Error = Infallible>,
{
    type Response = ();
    type Error = Infallible;

    async fn call(&self, req: usize) -> Result<Self::Response, Self::Error> {
        let old = self.counter.fetch_add(req, Ordering::AcqRel);
        let new = old + req;
        println!("SvcB called! {old}->{new}");
        self.inner.call(()).await?;
        Ok(())
    }
}

struct SvcBFactory<T>(T);

impl<T> MakeService for SvcBFactory<T>
where
    T: MakeService<Error = Infallible>,
{
    type Service = SvcB<T::Service>;
    type Error = Infallible;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(match old {
            Some(r) => SvcB {
                counter: r.counter.load(Ordering::Acquire).into(),
                inner: self.0.make_via_ref(Some(&r.inner))?,
            },
            None => SvcB {
                counter: 0.into(),
                inner: self.0.make()?,
            },
        })
    }
}

/// For simple logic, we can impl the Service and NewService for the same struct.
/// Which means the Service itself can be a factory.
struct SvcC<T> {
    inner: T,
}

impl<T, I> Service<I> for SvcC<T>
where
    T: Service<I, Error = Infallible>,
{
    type Response = ();
    type Error = Infallible;

    async fn call(&self, req: I) -> Result<Self::Response, Self::Error> {
        println!("SvcC called!");
        self.inner.call(req).await?;
        Ok(())
    }
}

impl<F> MakeService for SvcC<F>
where
    F: MakeService<Error = Infallible>,
{
    type Service = SvcC<F::Service>;
    type Error = Infallible;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Infallible> {
        Ok(SvcC {
            inner: self.inner.make_via_ref(old.map(|x| &x.inner))?,
        })
    }
}

// ===== impl layer fn for Factory instead of defining manually =====

impl SvcAFactory {
    fn layer<C>() -> impl FactoryLayer<C, (), Factory = Self>
    where
        C: Param<InitFlag>,
    {
        layer_fn(|c: &C, ()| SvcAFactory {
            init_flag: c.param(),
        })
    }
}

impl<T> SvcBFactory<T> {
    fn layer<C>() -> impl FactoryLayer<C, T, Factory = Self> {
        layer_fn(|_: &C, inner| SvcBFactory(inner))
    }
}

impl<T> SvcC<T> {
    fn layer<C>() -> impl FactoryLayer<C, T, Factory = Self> {
        layer_fn(|_: &C, inner| SvcC { inner })
    }

    fn opt_layer<C>(enabled: bool) -> Option<impl FactoryLayer<C, T, Factory = Self>> {
        if enabled {
            Some(layer_fn(|_: &C, inner| SvcC { inner }))
        } else {
            None
        }
    }
}

// ===== Define Config and impl Param<T> for it =====
#[derive(Clone, Copy)]
struct Config {
    init_flag: bool,
}

impl Param<InitFlag> for Config {
    fn param(&self) -> InitFlag {
        InitFlag(self.init_flag)
    }
}

#[main_macro]
async fn main() {
    let config = Config { init_flag: false };
    let stack = FactoryStack::new(config)
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer())
        // with Either, we can control whether using a layer at runtime
        .push(SvcC::opt_layer(true));
    let svc = stack.make().unwrap();
    svc.call(1).await.unwrap();
    svc.call(2).await.unwrap();
    svc.call(3).await.unwrap();

    // with BoxService, we can erase different types
    let boxed_svc: BoxedService<usize, (), _> = stack.push_boxed_service().make().unwrap();
    boxed_svc.call(1).await.unwrap();

    let config = Config { init_flag: true };
    let new_stack = FactoryStack::new(config)
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer())
        .push(SvcC::opt_layer(false))
        .into_inner();
    // create new service with new stack and old service
    let new_svc = new_stack.make_via_ref(Some(&svc)).unwrap();
    new_svc.call(10).await.unwrap();

    // also, BoxService can use it in this way too
    let new_svc = new_stack.make_via_ref(boxed_svc.downcast_ref()).unwrap();
    new_svc.call(10).await.unwrap();

    // to make it more flexible, we can even make the factory a boxed type.
    // so we can insert different layers and get a same type.
    #[allow(unused_assignments)]
    let mut fac: BoxedMakeService<_, _> = FactoryStack::new(config)
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer())
        .push_boxed_service()
        .push_boxed_factory()
        .into_inner();
    fac = FactoryStack::new(config)
        .push(SvcAFactory::layer())
        .push(SvcBFactory::layer())
        .push(SvcC::layer())
        .push_boxed_service()
        .push_boxed_factory()
        .into_inner();
    let svc = fac.make().unwrap();
    svc.call(1).await.unwrap();
}
