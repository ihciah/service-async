# Service Async
[![Crates.io](https://img.shields.io/crates/v/service-async.svg)](https://crates.io/crates/service-async)

A `Service` trait similar to tower-service [https://docs.rs/tower/latest/tower/trait.Service.html], in pure async style

## Motivation: Overcoming Limitations in Tower's Service Model

The Tower framework's `Service` trait, while powerful, presents some challenges:

1. Limited Capture Scope: As a future factory used serially and spawned for parallel execution, Tower's `Service` futures cannot capture `&self` or `&mut self`. This necessitates cloning and moving ownership into the future.

2. Complex Poll-Style Implementation: Tower's `Service` trait is defined in a poll-style, requiring manual state management. This often leads to verbose implementations using `Box<Pin<...>>` to leverage async/await syntax.

These limitations often result in code patterns like:

```rust
impl<S, Req> tower::Service<Req> for SomeStruct<S>
where
    // ...
{
    type Response = // ...;
    type Error = // ...;
    type Future = Pin<Box<dyn Future<Output = ...> + Send + 'static>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    
    fn call(&mut self, req: Req) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            client.get(req).await;
            // ...
        })
    }
}
```
## Introducing a Refined Service Trait

This crate leverages `impl Trait` to introduce a new `Service` trait, designed to simplify implementation and improve performance:

1. Efficient Borrowing: By using `impl Trait` in the return position, futures can now capture `&self` or `&mut self`, eliminating unnecessary cloning.

2. Zero-Cost Abstractions: Utilizing `impl Trait` instead of `Box<dyn...>` allows for more inline code optimization, especially for operations not crossing await points.

This approach combines the power of `impl Trait` with a refined `Service` trait to offer both flexibility and performance improvements.

To enable parallel execution with this new design, we propose two approaches:

1. Shared Immutable Access: Use `&self` with a single `Service` instance.
2. Exclusive Mutable Access: Use `&mut self` and create a new `Service` instance for each call.

The first approach is generally preferred, as mutable `Service` instances are often unnecessary for single-use scenarios.

Our refined [`Service`](https://docs.rs/service-async/latest/service_async/trait.Service.html) trait is defined as:

```rust
pub trait Service<Request> {
    /// Responses given by the service.
    type Response;
    /// Errors produced by the service.
    type Error;
    /// Process the request and return the response asynchronously.
    fn call(&self, req: Request) -> impl Future<Output = Result<Self::Response, Self::Error>>;
}
```

This design eliminates the need for a `poll_ready` function, as state is maintained within the future itself.

## Key Differences and Advantages

Compared to Tower's approach, our `Service` trait represents a paradigm shift:

- Role: It functions as a request handler rather than a future factory.
- State Management: Mutable state requires explicit synchronization primitives like `Mutex` or `RefCell`.
- Resource Efficiency: Our approach maintains reference relationships, incurring costs only when mutability is required, unlike Tower's shared ownership model where each share has an associated cost.

This refined `Service` trait offers a more intuitive, efficient, and flexible approach to building asynchronous services in Rust.

## MakeService

The `MakeService` trait provides a flexible way to construct service chains while allowing state migration from previous instances. This is particularly useful when services manage stateful resources like connection pools, and you need to update the service chain with new configurations while preserving existing resources.

Key features:

- `make_via_ref` method allows creating a new service while optionally referencing an existing one.
- Enables state preservation and resource reuse between service instances.
- `make` method provides a convenient way to create a service without an existing reference.

Example usage:

```rust
struct SvcA {
    pass_flag: bool,
    not_pass_flag: bool,
}

struct InitFlag(bool);

struct SvcAFactory {
    init_flag: InitFlag,
}

impl MakeService for SvcAFactory {
    type Service = SvcA;
    type Error = Infallible;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(match old {
            // SvcAFactory can access state from the older service
            // which was created. 
            Some(r) => SvcA {
                pass_flag: r.pass_flag,
                not_pass_flag: self.init_flag.0,
            },
            // There was no older service, so create SvcA from
            // service factory config.
            None => SvcA {
                pass_flag: self.init_flag.0,
                not_pass_flag: self.init_flag.0,
            },
        })
    }
}
```

This approach allows for efficient updates to service chains, preserving valuable resources when reconfiguring services.

# Service Factories and Composition

## Service Factories

In complex systems, creating and managing services often requires more flexibility than a simple constructor can provide. This is where the concept of Service factories comes into play. A Service factory is responsible for creating instances of services, potentially with complex initialization logic or state management.

## MakeService Trait

The `MakeService` trait is the cornerstone of our Service factory system. It provides a flexible way to construct service chains while allowing state migration from previous instances. This is particularly useful when services manage stateful resources like connection pools, and you need to update the service chain with new configurations while preserving existing resources.

Key features of `MakeService`:

- `make_via_ref` method allows creating a new service while optionally referencing an existing one.
- Enables state preservation and resource reuse between service instances.
- `make` method provides a convenient way to create a service without an existing reference.

Example usage:

```rust
struct SvcA {
    pass_flag: bool,
    not_pass_flag: bool,
}

struct InitFlag(bool);

struct SvcAFactory {
    init_flag: InitFlag,
}

impl MakeService for SvcAFactory {
    type Service = SvcA;
    type Error = Infallible;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(match old {
            // SvcAFactory can access state from the older service
            // which was created. 
            Some(r) => SvcA {
                pass_flag: r.pass_flag,
                not_pass_flag: self.init_flag.0,
            },
            // There was no older service, so create SvcA from
            // service factory config.
            None => SvcA {
                pass_flag: self.init_flag.0,
                not_pass_flag: self.init_flag.0,
            },
        })
    }
}
```

This approach allows for efficient updates to service chains, preserving valuable resources when reconfiguring services.

## FactoryLayer

To enable more complex service compositions, we introduce the concept of `FactoryLayer`. `FactoryLayer` is a trait that defines how to wrap one factory with another, creating a new composite factory. This allows for the creation of reusable, modular pieces of functionality that can be easily combined.

To simplify chain assembly, factories can define a `layer` function that creates a factory wrapper. This concept is similar to the Tower framework's `Layer`, but with a key difference:

1. Tower's `Layer`: Creates a `Service` wrapping an inner `Service`.
2. Our `layer`: Creates a `Factory` wrapping an inner `Factory`, which can then be used to create the entire `Service` chain.

## FactoryStack

`FactoryStack` is a powerful abstraction that allows for the creation of complex service chains. It manages a stack of service factories, providing methods to push new layers onto the stack and to create services from the assembled stack.

The `FactoryStack` works by composing multiple `FactoryLayer`s together. Each layer in the stack wraps the layers below it, creating a nested structure of factories. When you call `make` or `make_async` on a `FactoryStack`, it traverses this structure from the outermost layer to the innermost, creating the complete service chain.

This approach allows users to create complex service factories by chaining multiple factory layers together in an intuitive manner. Each layer can add its own functionality, modify the behavior of inner layers, or even completely transform the service chain.

To create a chain of services using `FactoryStack`:

1. Start with a `FactoryStack` initialized with your configuration.
2. Use the `push` method to add layers to the stack.
3. Each layer can modify or enhance the behavior of the inner layers.
4. Finally, call `make` or `make_async` to create the complete service chain.

This system offers a powerful and flexible way to construct and update service chains while managing state and resources efficiently. It allows for modular, reusable pieces of functionality, easy reconfiguration of service chains, and clear separation of concerns between different parts of your service logic.

## Putting it all together

This example demonstrates the practical application of the `MakeService`, `FactoryLayer`, and `FactoryStack` concepts. It defines several services (`SvcA` and `SvcB`) and their corresponding factories. The `FactoryStack` is then used to compose these services in a layered manner. The `Config` struct provides initial configuration, which is passed through the layers. Finally, in the `main` function, a service stack is created, combining `SvcAFactory` and `SvcBFactory`. The resulting service is then called multiple times, showcasing how the chain of services handles requests and maintains state.

For a more comprehensive example that further illustrates these concepts and their advanced usage, readers are encouraged to examine the `demo.rs` file in the examples directory of the project.

```rust
use std::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
};

use service_async::{
    layer::{layer_fn, FactoryLayer},
    stack::FactoryStack,
    AsyncMakeService, BoxedMakeService, BoxedService, MakeService, Param, Service,
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

// Implement Service trait for SvcA
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
            // SvcAFactory can access state from the older service
            // which was created. 
            Some(r) => SvcA {
                pass_flag: r.pass_flag,
                not_pass_flag: self.init_flag.0,
            },
            // There was no older service, so create SvcA from
            // service factory config.
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
        .push(SvcBFactory::layer());

    let svc = stack.make_async().await.unwrap();
    svc.call(1).await.unwrap();
    svc.call(2).await.unwrap();
    svc.call(3).await.unwrap();
}

```
