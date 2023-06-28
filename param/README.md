# Param

Param-style traits. Helpful in decoupling specific struct definitions.

```rust
pub trait Param<T> {
    fn param(&self) -> T;
}
```