use std::{future::Future, sync::Arc};

/// A trait implemented by service factories to create instances of services that implement the [`Service`](crate::Service) trait.
///
/// `MakeService` enables flexible service chain construction with state migration between instances.
/// It's particularly useful for managing stateful resources (e.g., connection pools) when updating
/// service configurations.
///
/// # Key Features
///
/// - State preservation and resource reuse between service instances
/// - Optional creation of services from scratch
/// - Efficient management of stateful resources across service updates
///
/// # Example Implementation
///
/// ```rust
/// use std::convert::Infallible;
/// use service_async::{MakeService, Service};
///
/// struct MyService {
///     connection_pool: ConnectionPool,
///     config: Config,
/// }
///
/// struct MyServiceFactory {
///     config: Config,
/// }
///
/// impl MakeService for MyServiceFactory {
///     type Service = MyService;
///     type Error = Infallible;
///
///     fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
///         Ok(match old {
///             Some(existing) => MyService {
///                 connection_pool: existing.connection_pool.clone(),
///                 config: self.config.clone(),
///             },
///             None => MyService {
///                 connection_pool: ConnectionPool::new(),
///                 config: self.config.clone(),
///             },
///         })
///     }
/// }
/// ```
///
/// In this example, `MyServiceFactory` implements `MakeService` to create `MyService` instances,
/// demonstrating how to reuse a connection pool when an existing service is provided.
pub trait MakeService {
    /// The type of service this factory creates.
    type Service;

    /// The type of error that can occur during service creation.
    type Error;

    /// Creates a new service, optionally using an existing service as a reference.
    ///
    /// This method allows for sophisticated service creation logic that can reuse
    /// state from an existing service instance.
    ///
    /// # Arguments
    ///
    /// * `old` - An optional reference to an existing service instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the newly created service or an error.
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error>;

    /// Creates a new service without referencing an existing one.
    ///
    /// This is a convenience method that calls `make_via_ref` with `None`.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the newly created service or an error.
    fn make(&self) -> Result<Self::Service, Self::Error> {
        self.make_via_ref(None)
    }
}

impl<T: MakeService + ?Sized> MakeService for &T {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        (*self).make_via_ref(old)
    }
}

impl<T: MakeService + ?Sized> MakeService for Arc<T> {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old)
    }
}

impl<T: MakeService + ?Sized> MakeService for Box<T> {
    type Service = T::Service;
    type Error = T::Error;
    fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old)
    }
}

/// A boxed trait object of `MakeService` that enables type erasure for service factories.
///
/// `BoxedMakeService<S, E>` allows different implementations of `MakeService` to be
/// treated uniformly, as long as they produce the same `Service` type `S` and `Error` type `E`.
//
/// This type is particularly useful when designing systems with pluggable or configurable
/// service factories, where the exact implementation of the factory may vary or be determined at runtime.
pub type BoxedMakeService<S, E> =
    Box<dyn MakeService<Service = S, Error = E> + Send + Sync + 'static>;

/// `ArcMakeService<S, E>` is similar to `BoxedMakeService<S, E>`, but uses `Arc` instead of `Box`.
/// This allows for multiple owners of the same `MakeService` implementation, enabling efficient
/// cloning and sharing across multiple components or threads.
///
/// # Key Features
///
/// - Shared Ownership: Allows multiple components to share the same `MakeService` instance.
/// - Type Erasure: Enables storing different `MakeService` implementations uniformly.
/// - Efficient Cloning: `Arc` allows for cheap cloning of the service factory reference.
///
/// This type is particularly useful in scenarios where a service factory needs to be
/// shared across multiple parts of an application, such as in worker pools.
pub type ArcMakeService<S, E> =
    Arc<dyn MakeService<Service = S, Error = E> + Send + Sync + 'static>;

pub type BoxedMakeBoxedService<Req, Resp, SE, ME> =
    BoxedMakeService<crate::BoxedService<Req, Resp, SE>, ME>;
pub type ArcMakeBoxedService<Req, Resp, SE, ME> =
    ArcMakeService<crate::BoxedService<Req, Resp, SE>, ME>;

/// A trait implemented by asynchronous service factories to create instances of services
/// that implement the [`Service`](crate::Service) trait.
///
/// `AsyncMakeService` is the asynchronous counterpart to [`MakeService`]. It enables flexible
/// service chain construction with state migration between instances, allowing for asynchronous
/// initialization or resource acquisition.
///
/// # Key Features
///
/// - Asynchronous service creation, suitable for I/O-bound initialization
/// - State preservation and resource reuse between service instances
/// - Optional creation of services from scratch
/// - Efficient management of stateful resources across service updates
///
/// # Example Implementation
///
/// ```rust
/// use std::convert::Infallible;
/// use your_crate::{AsyncMakeService, Service};
///
/// struct MyAsyncService {
///     connection_pool: AsyncConnectionPool,
///     config: Config,
/// }
///
/// struct MyAsyncServiceFactory {
///     config: Config,
/// }
///
/// impl AsyncMakeService for MyAsyncServiceFactory {
///     type Service = MyAsyncService;
///     type Error = Infallible;
///
///     async fn make_via_ref(&self, old: Option<&Self::Service>) -> Result<Self::Service, Self::Error> {
///         Ok(match old {
///             Some(existing) => MyAsyncService {
///                 connection_pool: existing.connection_pool.clone(),
///                 config: self.config.clone(),
///             },
///             None => MyAsyncService {
///                 connection_pool: AsyncConnectionPool::new().await,
///                 config: self.config.clone(),
///             },
///         })
///     }
/// }
/// ```
///
/// In this example, `MyAsyncServiceFactory` implements `AsyncMakeService` to create `MyAsyncService`
/// instances asynchronously, demonstrating how to reuse a connection pool or create a new one when needed.
pub trait AsyncMakeService {
    /// The type of service this factory creates.
    type Service;

    /// The type of error that can occur during service creation.
    type Error;

    /// Asynchronously creates a new service, optionally using an existing service as a reference.
    ///
    /// This method allows for sophisticated asynchronous service creation logic that can reuse
    /// state from an existing service instance.
    ///
    /// # Arguments
    ///
    /// * `old` - An optional reference to an existing service instance.
    ///
    /// # Returns
    ///
    /// A `Future` that resolves to a `Result` containing either the newly created service or an error.
    fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> impl Future<Output = Result<Self::Service, Self::Error>>;

    /// Asynchronously creates a new service without referencing an existing one.
    ///
    /// This is a convenience method that calls `make_via_ref` with `None`.
    ///
    /// # Returns
    ///
    /// A `Future` that resolves to a `Result` containing either the newly created service or an error.
    fn make(&self) -> impl Future<Output = Result<Self::Service, Self::Error>> {
        self.make_via_ref(None)
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for &T {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        (*self).make_via_ref(old).await
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for Arc<T> {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old).await
    }
}

impl<T: AsyncMakeService + ?Sized> AsyncMakeService for Box<T> {
    type Service = T::Service;
    type Error = T::Error;
    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        self.as_ref().make_via_ref(old).await
    }
}

/// Impl AsyncMakeService where T: MakeService.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncMakeServiceWrapper<T>(pub T);

impl<T: MakeService> AsyncMakeService for AsyncMakeServiceWrapper<T> {
    type Service = <T as MakeService>::Service;
    type Error = <T as MakeService>::Error;

    async fn make_via_ref(
        &self,
        old: Option<&Self::Service>,
    ) -> Result<Self::Service, Self::Error> {
        <T as MakeService>::make_via_ref(&self.0, old)
    }
    async fn make(&self) -> Result<Self::Service, Self::Error> {
        <T as MakeService>::make(&self.0)
    }
}
