// Attack: 100% safe user code. No `update` is even needed for the first case.
#![forbid(unsafe_code)]
use dfr::{Cell, Component};
static S: u64 = 7;
static SR: &'static u64 = &S;

// Field type `&'a u64`, Output `&'static u64`: same type id once lifetimes are erased.
struct Bad<'a> { r: &'a u64 }
impl<'a> Component for Bad<'a> {
    type Input<'x> = () where Self: 'x;
    type Output = &'static u64;
    fn update<'s>(&'s mut self, _: ()) -> &'s &'static u64 { &SR }
}
fn launder() -> &'static u64 {
    let local = Box::new(42u64);
    let cell = Cell::new(Bad { r: &*local });
    *cell.view()          // a `&'static u64` that really points at `local`
}

// One nesting level: no direct usize field, so it finds Vec's private length.
struct V { v: Vec<u64> }
static ZERO: usize = 0;
impl Component for V {
    type Input<'x> = () where Self: 'x;
    type Output = usize;
    fn update<'s>(&'s mut self, _: ()) -> &'s usize { self.v.push(1); &ZERO }
}

fn main() {
    let v = Cell::new(V { v: vec![1, 2, 3] });
    println!("nested field found through Vec: {}", v.view());
    let r = launder();
    let noise = vec![999_999u64; 4];
    println!("laundered: {}  noise {:?}", *r, noise);
}
