//! Run with `cargo run -p dataflow-flat --example basic`.
//!
//! Text pipeline: one root (`Line`, the event fed with `push`), components that
//! subscribe to it and to each other, and a sink (`Printer`). Everything
//! upstream of the sinks is found, built once, and run in order on every push.
//! (Phase 1a of the redesign: the examples get a proper rewrite in phase 2.)

mod components;

use components::{Line, Printer, Tokenize, TokenizeConfig, TopWords, TopWordsConfig};
use dataflow_flat::{Configs, Pipeline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configs for the components that need one. A component without an entry
    // gets `Config::default()`. Checked when the pipeline is built: an entry
    // for a component that is not in the graph is an error.
    let configs = Configs::new()
        .with::<Tokenize>(TokenizeConfig { lowercase: true })
        .with::<TopWords>(TopWordsConfig { n: 2 });

    let mut pipeline = Pipeline::<Line, (Printer,)>::build(configs)?;

    println!("invocation order:");
    for (i, name) in pipeline.order().iter().enumerate() {
        println!("  {i}. {}", name.replace("basic::components::", ""));
    }

    for line in ["the quick brown fox jumps over the lazy dog", "The dog sleeps and the fox runs"] {
        // Every component runs once, in that order; the sink prints.
        pipeline.push(Line(line.to_string()));
    }

    Ok(())
}
