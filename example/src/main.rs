//! Text pipeline: name only the roots; everything they subscribe to,
//! directly or indirectly, is found, built once, and run in order.

mod components;

use components::{Len, Report, Source, SourceConfig, Tokenize, TokenizeConfig, TopWords, TopWordsConfig};
use dataflow_flat::{deps, Configs, Deps, Pipeline};

/// The components whose results this program wants. Source, Tokenize,
/// WordCount, TopWords and Longest are discovered from subscriptions.
type Roots = Deps![Report, Len<Tokenize>];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One owned config per component without a default. Checked when the
    // pipeline is built: nothing missing, nothing for components not in the graph.
    let configs = Configs::new()
        .with::<Source>(SourceConfig {
            lines: vec![
                "the quick brown fox jumps over the lazy dog".into(),
                "The dog sleeps and the fox runs".into(),
            ],
        })
        .with::<Tokenize>(TokenizeConfig { lowercase: true })
        .with::<TopWords>(TopWordsConfig { n: 2 });

    let mut pipeline = Pipeline::<Roots>::build(configs)?;

    println!("invocation order:");
    for (i, name) in pipeline.order().iter().enumerate() {
        println!("  {i}. {}", name.replace("example::components::", ""));
    }

    for run in 1..=2 {
        pipeline.run();
        // Root outputs, in the order the roots are listed.
        let deps!(report, token_count) = pipeline.outputs().unwrap();
        println!("\nrun {run}:");
        println!("  {report}");
        println!("  token count: {token_count}");
    }

    Ok(())
}
