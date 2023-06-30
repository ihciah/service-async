#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
// #![feature(async_fn_in_trait)]

// use std::{cell::UnsafeCell, future::Future, rc::Rc};

use async_trait::async_trait;

#[async_trait]
pub trait Service<Request>: Sync + Send {
    type Response;
    //     /// Errors produced by the service.
    type Error;
    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error>;
}

struct A;
#[async_trait]
impl Service<String> for A {
    type Response = String;
    type Error = ();

    async fn call(&self, req: String) -> Result<Self::Response, Self::Error> {
        Ok(format!("A -> {}", req))
    }
}

struct B<S> {
    inner: S,
}
#[async_trait]
impl<S> Service<String> for B<S>
where
    S: Service<String, Response = String, Error = ()>,
{
    type Response = String;
    type Error = ();
    async fn call(&self, req: String) -> Result<Self::Response, Self::Error> {
        let req = format!("B -> {}", req);
        self.inner.call(req).await
    }
}

pub trait Layer<S> {
    type Service;
    fn layer(&self, inner: S) -> Self::Service;
}
struct LayerB;
impl<S> Layer<S> for LayerB
where
    S: Service<String>,
{
    type Service = B<S>;
    fn layer(&self, inner: S) -> Self::Service {
        B { inner }
    }
}

#[monoio::main]
async fn main() {
    let a = A;
    let layer = LayerB;
    let b = layer.layer(a);
    let mut c = Vec::<Box<dyn Service<String, Response = String, Error = ()>>>::new();
    c.push(Box::new(A));

    let result = b.call(String::from("abcdfdssfd")).await;
    println!("{:?}", result);
    for d in c {
        println!("d result: {:?}", d.call(String::from("abcd")).await)
    }
}
