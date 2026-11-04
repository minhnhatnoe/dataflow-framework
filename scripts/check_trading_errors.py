#!/usr/bin/env python3
"""Compile-fail checks for the trading layer. Run with the repository resource cap."""
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent.parent
example = (root / "dataflow-trading/examples/strategy.rs").read_text()
book_impl = """
impl Book for Bad {
    type Config=(); type State=f64;
    fn new(_:())->(Self,f64) { (Self,0.0) }
    fn update(&mut self,_:Self::Inputs<'_>,_: &mut f64) {}
}
"""
signal_impl = """
impl Signal for Bad {
    type Config=(); type Output=FairOutput;
    fn new(_:())->Self { Self }
    fn compute(&mut self,_:Self::Inputs<'_>)->FairOutput { FairOutput { price:0.0,confidence:0.0 } }
}
"""
oms_impl = """
impl Oms for Bad {
    type Config=(); fn new(_:())->Self { Self }
    fn compute(&mut self,_:Self::Inputs<'_>,_:Quotes<'_,VenueA>)->Vec<Order> { vec![] }
}
"""
cases = {
    "non_float_output": ('#[derive(SignalOutputs)] struct Bad { pub value:u64 }', 'signal output fields must be public f64 values'),
    "private_output": ('#[derive(SignalOutputs)] struct Bad { value:f64 }', 'signal output fields must be public f64 values'),
    "unknown_field": ('#[signal] #[inputs(value: Fair<VenueA,Btc> => missing)] struct Bad;' + signal_impl, 'E0609'),
    "duplicate_input": ('#[signal] #[inputs(x: Fair<VenueA,Btc>, x: Fair<VenueA,Btc>)] struct Bad;', 'duplicate input name'),
    "source_on_signal": ('#[signal] #[source(event: Tick)] struct Bad;', 'only Books may declare one root source'),
    "non_root_source": ('#[book] #[source(event: Fair<VenueA,Btc>)] struct Bad;' + book_impl, 'Root'),
    "book_reads_signal": ('#[book] #[inputs(value: Fair<VenueA,Btc>)] struct Bad;' + book_impl, 'BookNode'),
    "oms_reads_signal": ('#[oms(exchange=VenueA)] #[inputs(value: Fair<VenueA,Btc>)] #[quotes(btc: Quoter<VenueA,Btc>)] struct Bad;' + oms_impl, 'BookNode'),
    "signal_reads_quote": ('#[signal] #[inputs(value: Quoter<VenueA,Btc>)] struct Bad;' + signal_impl, 'FeatureNode'),
    "missing_exchange": ('#[quote_engine(symbol=Btc)] struct Bad;', 'exchange is required'),
    "wrong_exchange": ('struct Other; impl Exchange for Other { type Order=Order; } #[oms(exchange=VenueA)] #[quotes(btc: Quoter<Other,Btc>)] struct Bad;' + oms_impl, 'E0271'),
    "empty_quotes": ('#[oms(exchange=VenueA)] #[quotes()] struct Bad;', 'an OMS must declare its quote set'),
}
base = root / "target/trading-errors"
base.mkdir(parents=True, exist_ok=True)
for name, (extra, expected) in cases.items():
    folder = base / name
    (folder / "src").mkdir(parents=True, exist_ok=True)
    (folder / "Cargo.toml").write_text(f'''[package]
name = "trading-error-{name.replace('_', '-')}"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
dataflow-flat = {{ path = "{root / 'dataflow-flat'}" }}
dataflow-trading = {{ path = "{root / 'dataflow-trading'}" }}
''')
    (folder / "src/main.rs").write_text(example + "\n" + extra)
    result = subprocess.run(["cargo", "check", "--offline", "--quiet"], cwd=folder,
        env={**os.environ, "CARGO_TARGET_DIR": str(base / "build"), "CARGO_BUILD_JOBS": "1"},
        capture_output=True, text=True, timeout=90)
    (folder / "output.log").write_text(result.stdout + result.stderr)
    assert result.returncode != 0 and expected in result.stderr, f"{name}: expected {expected}; see {folder / 'output.log'}"
    print(f"PASS {name}", flush=True)
