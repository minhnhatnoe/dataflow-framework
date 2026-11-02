use analyzeme::ProfilingData;
use std::path::Path;
// usage: keys FILE LABEL N  -> top N events of LABEL by inclusive duration, with args
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let n: usize = a[3].parse().unwrap();
    let mut v = vec![];
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if f.label == a[2].as_str() {
            let dur = e.duration().map(|x| x.as_secs_f64() * 1e3).unwrap_or(0.0);
            v.push((dur, f.additional_data.join(" | ")));
        }
    }
    v.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());
    println!("count {}", v.len());
    for (t, s) in v.iter().take(n) {
        let s: String = s.chars().take(260).collect();
        println!("{:.1}ms {}", t, s);
    }
}
