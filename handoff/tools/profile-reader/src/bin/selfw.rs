use analyzeme::{ProfilingData, Timestamp};
use std::{collections::HashMap, path::Path, time::SystemTime};
// usage: selfw FILE KEYSUBSTR -> self time per (label, key) inside the largest check_well_formed whose key contains KEYSUBSTR
fn key_of(ad: &str) -> String {
    match ad.find("DefId(") {
        Some(i) => {
            let r = &ad[i + 6..];
            let j = r.find(')').unwrap_or(r.len());
            let t = &r[..j];
            t.split(" ~ ").nth(1).unwrap_or(t).chars().take(110).collect()
        }
        None => ad.chars().take(110).collect(),
    }
}
type Fr = (SystemTime, SystemTime, String, String, f64);
fn fin(s: Fr, m: &mut HashMap<(String, String), (u64, f64)>) -> f64 {
    let dur = s.1.duration_since(s.0).unwrap().as_secs_f64() * 1e3;
    let en = m.entry((s.2, s.3)).or_insert((0, 0.0));
    en.0 += 1;
    en.1 += dur - s.4;
    dur
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let mut evs: Vec<(SystemTime, SystemTime, u32, String, String)> = vec![];
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if let analyzeme::EventPayload::Timestamp(Timestamp::Interval { start, end }) = f.payload {
            evs.push((start, end, f.thread_id, f.label.to_string(), key_of(&f.additional_data.join(" | "))));
        }
    }
    let win = evs
        .iter()
        .filter(|e| e.3 == "check_well_formed" && e.4.contains(&a[2]))
        .max_by_key(|e| e.1.duration_since(e.0).unwrap())
        .unwrap()
        .clone();
    println!("window {:?} {}", win.1.duration_since(win.0).unwrap(), win.4);
    let mut v: Vec<_> = evs.into_iter().filter(|e| e.2 == win.2 && e.0 >= win.0 && e.1 <= win.1).collect();
    v.sort_by(|x, y| x.0.cmp(&y.0).then(y.1.cmp(&x.1)));
    let mut selft: HashMap<(String, String), (u64, f64)> = HashMap::new();
    let mut stack: Vec<Fr> = vec![];
    for e in v {
        while let Some(t) = stack.last() {
            if t.1 <= e.0 {
                let s = stack.pop().unwrap();
                let dur = fin(s, &mut selft);
                if let Some(p) = stack.last_mut() { p.4 += dur; }
            } else {
                break;
            }
        }
        stack.push((e.0, e.1, e.3, e.4, 0.0));
    }
    while let Some(s) = stack.pop() {
        let dur = fin(s, &mut selft);
        if let Some(p) = stack.last_mut() { p.4 += dur; }
    }
    let mut l: Vec<_> = selft.into_iter().collect();
    l.sort_by(|x, y| y.1 .1.partial_cmp(&x.1 .1).unwrap());
    let mut bylabel: HashMap<String, (u64, f64)> = HashMap::new();
    for ((lab, _), (c, t)) in &l {
        let e = bylabel.entry(lab.clone()).or_insert((0, 0.0));
        e.0 += c;
        e.1 += t;
    }
    let mut bl: Vec<_> = bylabel.into_iter().collect();
    bl.sort_by(|x, y| y.1 .1.partial_cmp(&x.1 .1).unwrap());
    println!("-- by label (calls, self ms)");
    for (k, (c, t)) in bl.iter().take(12) { println!("{k}\t{c}\t{t:.0}"); }
    println!("-- top keys");
    for ((lab, k), (c, t)) in l.iter().take(25) { println!("{lab}\t{c}\t{t:.0}\t{k}"); }
}
