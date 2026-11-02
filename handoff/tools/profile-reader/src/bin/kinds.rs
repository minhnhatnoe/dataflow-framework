use analyzeme::ProfilingData;
use std::{collections::HashMap, path::Path};
// usage: kinds FILE [N] -> non-query events by (kind,label): count, inclusive ms
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let n: usize = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(25);
    let mut m: HashMap<(String, String), (u64, f64)> = HashMap::new();
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if f.event_kind == "Query" || f.event_kind == "QueryCacheHit" { continue; }
        let dur = e.duration().map(|x| x.as_secs_f64() * 1e3).unwrap_or(0.0);
        let v = m.entry((f.event_kind.to_string(), f.label.to_string())).or_insert((0, 0.0));
        v.0 += 1;
        v.1 += dur;
    }
    let mut v: Vec<_> = m.into_iter().collect();
    v.sort_by(|x, y| y.1 .1.partial_cmp(&x.1 .1).unwrap());
    for ((k, l), (c, t)) in v.iter().take(n) { println!("{}\t{}\t{}\t{:.0}ms", k, l, c, t); }
}
