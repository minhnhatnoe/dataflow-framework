use analyzeme::ProfilingData;
use std::{collections::HashMap, path::Path};
// usage: cnt FILE [N] -> executed query events counted by label, top N, plus total
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let n: usize = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(25);
    let mut m: HashMap<String, (u64, f64)> = HashMap::new();
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if f.event_kind != "Query" { continue; }
        let dur = e.duration().map(|x| x.as_secs_f64() * 1e3).unwrap_or(0.0);
        let v = m.entry(f.label.to_string()).or_insert((0, 0.0));
        v.0 += 1; v.1 += dur;
    }
    let tot: u64 = m.values().map(|v| v.0).sum();
    println!("total query executions {}", tot);
    let mut v: Vec<_> = m.into_iter().collect();
    v.sort_by(|x, y| y.1 .0.cmp(&x.1 .0));
    for (l, (c, t)) in v.iter().take(n) { println!("{}\t{}\t{:.0}ms", l, c, t); }
}
