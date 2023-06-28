use std::{error::Error, fmt::Display, future::Future, pin::Pin};

use crate::{layer::FactoryLayer, MakeService, Service};

#[derive(Debug, Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<C, F, T> FactoryLayer<C, F> for Option<T>
where
    T: FactoryLayer<C, F>,
{
    type Factory = Either<T::Factory, F>;

    #[inline]
    fn layer(&self, config: &C, inner: F) -> Self::Factory {
        match self {
            Some(fl) => Either::Left(fl.layer(config, inner)),
            None => Either::Right(inner),
        }
    }
}

impl<C, F, FLA, FLB> FactoryLayer<C, F> for Either<FLA, FLB>
where
    FLA: FactoryLayer<C, F>,
    FLB: FactoryLayer<C, F>,
{
    type Factory = Either<FLA::Factory, FLB::Factory>;

    #[inline]
    fn layer(&self, config: &C, inner: F) -> Self::Factory {
        match self {
            Either::Left(fl) => Either::Left(fl.layer(config, inner)),
            Either::Right(fl) => Either::Right(fl.layer(config, inner)),
        }
    }
}

impl<A, B> MakeService for Either<A, B>
where
    A: MakeService,
    B: MakeService,
{
    type Service = Either<A::Service, B::Service>;
    type Error = Either<A::Error, B::Error>;

    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        match self {
            Either::Left(f) => match old.as_ref() {
                Some(Either::Left(left_svc)) => f.make_via_ref(Some(left_svc)),
                _ => f.make(),
            }
            .map(Either::Left)
            .map_err(Either::Left),
            Either::Right(f) => match old.as_ref() {
                Some(Either::Right(right_svc)) => f.make_via_ref(Some(right_svc)),
                _ => f.make(),
            }
            .map(Either::Right)
            .map_err(Either::Right),
        }
    }
}

impl<A, B, R> Service<R> for Either<A, B>
where
    A: Service<R>,
    B: Service<R, Response = A::Response, Error = A::Error>,
{
    type Response = A::Response;
    type Error = A::Error;
    type Future<'cx> = Either<A::Future<'cx>, B::Future<'cx>>
    where
        Self: 'cx,
        R: 'cx;

    #[inline]
    fn call(&self, req: R) -> Self::Future<'_> {
        match self {
            Either::Left(s) => Either::Left(s.call(req)),
            Either::Right(s) => Either::Right(s.call(req)),
        }
    }
}

impl<A, B> Future for Either<A, B>
where
    A: Future,
    B: Future<Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.as_pin_mut() {
            Either::Left(fut) => fut.poll(cx),
            Either::Right(fut) => fut.poll(cx),
        }
    }
}

// Copied from futures-util.
impl<A, B> Either<A, B> {
    /// Convert `Pin<&Either<A, B>>` to `Either<Pin<&A>, Pin<&B>>`,
    /// pinned projections of the inner variants.
    #[inline]
    pub fn as_pin_ref(self: Pin<&Self>) -> Either<Pin<&A>, Pin<&B>> {
        // SAFETY: We can use `new_unchecked` because the `inner` parts are
        // guaranteed to be pinned, as they come from `self` which is pinned.
        unsafe {
            match *Pin::get_ref(self) {
                Either::Left(ref inner) => Either::Left(Pin::new_unchecked(inner)),
                Either::Right(ref inner) => Either::Right(Pin::new_unchecked(inner)),
            }
        }
    }

    /// Convert `Pin<&mut Either<A, B>>` to `Either<Pin<&mut A>, Pin<&mut B>>`,
    /// pinned projections of the inner variants.
    #[inline]
    pub fn as_pin_mut(self: Pin<&mut Self>) -> Either<Pin<&mut A>, Pin<&mut B>> {
        // SAFETY: `get_unchecked_mut` is fine because we don't move anything.
        // We can use `new_unchecked` because the `inner` parts are guaranteed
        // to be pinned, as they come from `self` which is pinned, and we never
        // offer an unpinned `&mut A` or `&mut B` through `Pin<&mut Self>`. We
        // also don't have an implementation of `Drop`, nor manual `Unpin`.
        unsafe {
            match *Pin::get_unchecked_mut(self) {
                Either::Left(ref mut inner) => Either::Left(Pin::new_unchecked(inner)),
                Either::Right(ref mut inner) => Either::Right(Pin::new_unchecked(inner)),
            }
        }
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::Left(inner) => inner.fmt(f),
            Either::Right(inner) => inner.fmt(f),
        }
    }
}

impl<A: Error, B: Error> Error for Either<A, B> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Either::Left(inner) => inner.source(),
            Either::Right(inner) => inner.source(),
        }
    }
}

impl<T> Either<T, T> {
    #[inline]
    pub fn into_inner(self) -> T {
        match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        }
    }
}
