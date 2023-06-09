pub trait Param<T> {
    fn param(&self) -> T;
}

pub trait ParamRef<T> {
    fn param_ref(&self) -> &T;
}

pub trait ParamMut<T> {
    fn param_mut(&mut self) -> &mut T;
}

impl<T: Clone> Param<T> for T {
    fn param(&self) -> T {
        self.clone()
    }
}

impl<T> ParamRef<T> for T {
    fn param_ref(&self) -> &T {
        self
    }
}

impl<T> ParamMut<T> for T {
    fn param_mut(&mut self) -> &mut T {
        self
    }
}
