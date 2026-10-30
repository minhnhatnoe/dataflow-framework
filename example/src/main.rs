//! Text pipeline: define one root component (`App`) that subscribes to what
//! the program wants; everything it subscribes to, directly or indirectly, is
//! found, built once, and run in order. `run` returns the root's output.

mod components;

use components::{App, Source, SourceConfig, Tokenize, TokenizeConfig, TopWords, TopWordsConfig};
use dataflow_flat::{Configs, Pipeline};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configs for the components that need one. A component without an entry
    // gets `Config::default()`. Checked when the pipeline is built: an entry
    // for a component that is not in the graph is an error.
    let configs = Configs::new()
        .with::<Source>(SourceConfig {
            lines: vec![
                "the quick brown fox jumps over the lazy dog".into(),
                "The dog sleeps and the fox runs".into(),
            ],
        })
        .with::<Tokenize>(TokenizeConfig { lowercase: true })
        .with::<TopWords>(TopWordsConfig { n: 2 });

    let mut pipeline = Pipeline::<App>::build(configs)?;

    println!("invocation order:");
    for (i, name) in pipeline.order().iter().enumerate() {
        println!("  {i}. {}", name.replace("example::components::", ""));
    }

    for run in 1..=2 {
        // Every component runs once, in that order; the root's output comes
        // back by value.
        let summary = pipeline.run();
        println!("\nrun {run}:");
        println!("  {}", summary.report);
        println!("  token count: {}", summary.token_count);
    }

    Ok(())
}
