use analyzeme::ProfilingData;
use std::{collections::HashMap, path::Path};
// usage: agg FILE LABEL[,LABEL..] -> per stripped key: count, total ms, max ms
fn strip(s: &str) -> String {
    let mut o = String::new(); let mut d = 0;
    for c in s.chars() {
        match c { '<' => { if d == 0 { o.push_str("<..>"); } d += 1; }
                  '>' => { if d > 0 { d -= 1; } }
                  _ => if d == 0 { o.push(c); } }
    }
    o.chars().take(160).collect()
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let labels: Vec<&str> = a[2].split(',').collect();
    let mut m: HashMap<(String, String), (u64, f64, f64)> = HashMap::new();
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if !labels.contains(&&*f.label) { continue; }
        let dur = e.duration().map(|x| x.as_secs_f64() * 1e3).unwrap_or(0.0);
        let ad = f.additional_data.join(" | ");
        let key = match ad.find("DefId(") { Some(i) => { let r = &ad[i+6..]; let j = r.find(')').unwrap_or(r.len()); let t = &r[..j]; let t = t.split(" ~ ").nth(1).unwrap_or(t).to_string(); let t = if ad.contains("Codegen") {format!("[cg] {t}")} else {t}; t }, None => strip(&ad) };
        let k = (f.label.to_string(), key);
        let v = m.entry(k).or_insert((0, 0.0, 0.0));
        v.0 += 1; v.1 += dur; if dur > v.2 { v.2 = dur; }
    }
    let mut v: Vec<_> = m.into_iter().collect();
    v.sort_by(|x, y| y.1 .1.partial_cmp(&x.1 .1).unwrap());
    for ((l, k), (c, t, mx)) in v.iter().take(40) { println!("{}\t{}\t{:.1}\t{:.2}\t{}", l, c, t, mx, k); }
}
