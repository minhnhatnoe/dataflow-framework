#![forbid(unsafe_code)]
use dfr::{Cell, Component};
#[derive(Clone, Copy, Debug, Default)] pub struct Top { bid: u64, ask: u64 }
struct Trades { window: u64, count: u64, levels: Vec<u64>, top: Top }
impl Component for Trades {
    type Input<'x> = u64 where Self: 'x;
    type Output = Top;
    fn update<'s>(&'s mut self, px: u64) -> &'s Top { self.count += 1; self.window = px; self.levels.push(px); self.top.bid = px; &self.top }
}
fn main() {
    let mut t = Cell::new(Trades { window: 0, count: 0, levels: vec![], top: Top::default() });
    println!("never ran: {:?}", t.view());
    t.run(5); t.run(9);
    println!("ran: {:?}", t.view());
    let mut moved = Box::new(t); moved.run(11);
    let back = *moved;
    println!("moved, did not run: {:?}", back.view());
}
