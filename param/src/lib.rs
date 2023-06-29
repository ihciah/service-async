pub trait Param<T> {
    fn param(&self) -> T;
}

pub trait ParamRef<T> {
    fn param_ref(&self) -> &T;
}

pub trait ParamMaybeRef<T> {
    fn param_maybe_ref(&self) -> Option<&T>;
}

pub trait ParamMut<T> {
    fn param_mut(&mut self) -> &mut T;
}

pub trait ParamMaybeMut<T> {
    fn param_maybe_mut(&mut self) -> Option<&mut T>;
}

pub trait ParamSet<T> {
    type Transformed;
    fn param_set(self, item: T) -> Self::Transformed;
}
pub trait ParamRemove<T> {
    type Transformed;
    fn param_remove(self) -> Self::Transformed;
}

pub trait ParamTake<T> {
    type Transformed;
    fn param_take(self) -> (Self::Transformed, T);
}

impl<T: Clone> Param<T> for T {
    fn param(&self) -> T {
        self.clone()
    }
}
