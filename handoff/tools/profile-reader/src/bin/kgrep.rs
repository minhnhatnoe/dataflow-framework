use analyzeme::ProfilingData;
use std::path::Path;
// usage: kgrep FILE LABEL SUBSTR N MAXCHARS -> first N events of LABEL whose key contains SUBSTR, full args
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = ProfilingData::new(Path::new(&a[1])).unwrap();
    let n: usize = a[4].parse().unwrap();
    let mx: usize = a[5].parse().unwrap();
    let mut k = 0;
    for e in d.iter() {
        let f = d.to_full_event(&e);
        if f.label != a[2].as_str() { continue; }
        let s = f.additional_data.join(" | ");
        if s.contains(a[3].as_str()) {
            let s: String = s.chars().skip(200).take(mx).collect();
            println!("{}\n", s);
            k += 1;
            if k >= n { break; }
        }
    }
}
