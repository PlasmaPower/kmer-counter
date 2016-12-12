#[derive(PartialEq, Eq, Clone)]
pub enum JoinMethod {
    Concat,
    Join,
    Sort,
}

// TODO: Figure out how this works with Mutex
pub trait Collector {
    fn add(&mut self, count: (u64, u16));
    fn finalize(&mut self);
    fn subdivide(&mut self, join_method: JoinMethod) -> Box<&mut Collector>;
}
