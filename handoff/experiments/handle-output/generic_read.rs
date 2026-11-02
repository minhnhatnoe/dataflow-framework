// Probe: does a generic by-value component keep seeing `&Output` when the input field type is a
// projection `<X as Edge>::Read<'a>`?  (the patched library broke marketlib's `Cross<A, B>`.)
// rustc --crate-type lib --emit metadata generic_read.rs
pub trait Edge { type Read<'a> where Self: 'a; }
// by-value nodes: the supertrait pins `Read`
pub trait Node: Edge + 'static
where
    for<'a> Self: Edge<Read<'a> = &'a <Self as Node>::Output>,
{
    type Output: 'static;
}
pub struct Out(pub u64);
pub struct A;
impl Edge for A { type Read<'a> = &'a Out; }
impl Node for A { type Output = Out; }
// a handle component is an `Edge` but not a `Node`
pub struct H;
impl Edge for H { type Read<'a> = u64; }

pub struct Input<'a, X: Edge + 'a> { pub x: <X as Edge>::Read<'a> }
fn cross(a: &Out) -> u64 { a.0 }
// generic user code, written today with only `X: Node<Output = Out>`
pub fn generic<X: Node<Output = Out>>(i: Input<'_, X>) -> u64 { cross(i.x) }
pub fn concrete(i: Input<'_, A>) -> u64 { cross(i.x) }
pub fn handle(i: Input<'_, H>) -> u64 { i.x }
