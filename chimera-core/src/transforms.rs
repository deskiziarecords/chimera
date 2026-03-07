pub trait Transform<Input> {
    type Output;
    fn apply(&self, input: Input) -> Self::Output;
    fn name(&self) -> &'static str;
}

pub struct Grad<Input, Output> {
    f: BoxedFunction<Input, Output>,
    argnums: Vec<usize>,
}