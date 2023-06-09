# Service Async
[![Crates.io](https://img.shields.io/crates/v/service-async.svg)](https://crates.io/crates/service-async)

A `Service` like tower in async style.

## Why
In tower system, the `Service` is a future factory, we usually use it serially and spawn the future to make them running in parallel.
1. But in this style means the future cannot capture `&self` or `&mut self`. We have to clone and move ownership into the future.
2. Also, the `Service` trait of tower is defined in `poll` style, which means we have to maintain status by ourself. Writing `poll` is hard, usually we have to use `Box<Pin<...>>` to utilize async/await.

That's why we can see so many code like this:
```rust
impl<S, Req> tower::Service<Req> for SomeStruct<S>
where
    ...
{
    type Response = ...;
    type Error = ...;
    type Future = Pin<Box<dyn Future<Output = ...> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            client.get(req).await;
            ...
        })
    }
}
```

## Use Service-Async
With this crate, users can make their code simpler and faster. There's no unnecessary clone or `Box<dyn<...>>` here.

1. To avoid clone, we can make the future not static and capture `&self` or `&mut self` with GAT.
2. To avoid Box, we can utilize `impl_trait_in_assoc_type`. Without Box, more code can be inlined if they not cross an await point.

Now the future generated by the Service captures `&self` or `&mut self`, to make it can run in parallel, we have to choose from these 2 solutions:
1. Use `&self` and a single Service instance.
2. Use `&mut self` and create a new Service instance on a new call.

The solution1 seems better. Making Service itself mutable is useless when it is for one-time use.

So we get a new Service with GAT:
```rust
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
```
There's also no need for keeping a function like `poll_ready` since we maintain state inside the future.

Compared with tower, this Service is used in a completely different way. The Service is no longer a future factory but a request handler. It has to use Mutex or RefCell if users want mutable.

Tower's Service needs to use shared ownership to tear down the reference relationship (each share pays a cost), our Service keeps the reference relationship, and the user only pays the cost when they need mutable.

## Assemble Service
The `Layer` provided by tower is a good Service assembler, it does not couple the definition of Service trait. You can always use it if it meets your needs.

This crate also provides a way to assemble services with the ability to merge state from old service chain. It helps when old services maintain resources like connection pool, and users want to update the service chain with new configuration.

The factories that impl `MakeService` can create service via an optional old one. To make the chain easier to assemble, a factory can define a `layer` fn to create a factory wrapper. It works like tower `Layer`: tower's layer creates Service with inner Service; our layer creates Factory with inner Factory, and the Factory can be used to create the whole Service.

So tower's layer is not a recursive structure, as well as our factory layer. With the help of `FactoryStack`, users can create a factory by composing factory layers in chain style.

## Use Case
Demo example illustrates how this system works.

A common use case is a gateway app: the main thread receives updates and creates factory, and send the shared factory to worker threads. Worker threads create Service with the shared factory, then wrap it with `Rc` then replace the maintained one. When a new request comes, the `Rc<Svc>` will be cloned and used to process the request. With the help of this crate, updating and migrating service state become easy.

## Notes
This crate requires nightly toolchain.