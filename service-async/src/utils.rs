use std::convert::Infallible;

use super::MakeService;

#[derive(Debug, Clone)]
pub struct CloneFactory<T> {
    svc: T,
}

impl<T> MakeService for CloneFactory<T>
where
    T: Clone,
{
    type Service = T;

    type Error = Infallible;

    #[inline]
    fn make_via_ref(&self, _old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        Ok(self.svc.clone())
    }
}

impl<T> From<T> for CloneFactory<T> {
    #[inline]
    fn from(svc: T) -> Self {
        CloneFactory { svc }
    }
}

impl<T> CloneFactory<T> {
    #[inline]
    pub const fn new(svc: T) -> Self {
        Self { svc }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.svc
    }
}
