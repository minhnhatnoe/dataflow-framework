use std::collections::BTreeMap;
use std::marker::PhantomData;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Node, Root};

// --- Line: the root. One value per push; nothing of it is stored ------------

/// The event fed to the pipeline: a line of text.
#[derive(Root)]
pub struct Line(pub String);

// --- Tokenize: three components subscribe to it; it still runs once per push --

#[derive(Default)]
pub struct TokenizeConfig {
    pub lowercase: bool,
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(line: Line)]
pub struct Tokenize {
    config: TokenizeConfig,
}

impl Component for Tokenize {
    type Config = TokenizeConfig;
    type Output = Vec<String>;

    fn new(config: TokenizeConfig, _: Self::Init<'_>) -> (Self, Vec<String>) {
        (Tokenize { config }, Vec::new())
    }

    fn update(&mut self, input: Self::Input<'_>) -> Vec<String> {
        println!("    (Tokenize::update)");
        input
            .line
            .0
            .split_whitespace()
            .map(|w| if self.config.lowercase { w.to_lowercase() } else { w.to_string() })
            .collect()
    }
}

// --- WordCount: unit config (`()` is `Default`), so it needs no entry in Configs

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(tokens: Tokenize)]
pub struct WordCount;

impl Component for WordCount {
    type Config = ();
    type Output = BTreeMap<String, usize>;

    fn new((): (), _: Self::Init<'_>) -> (Self, Self::Output) {
        (WordCount, BTreeMap::new())
    }

    fn update(&mut self, input: Self::Input<'_>) -> Self::Output {
        let mut counts = BTreeMap::new();
        for t in input.tokens {
            *counts.entry(t.clone()).or_insert(0) += 1;
        }
        counts
    }
}

// --- TopWords --------------------------------------------------------------

#[derive(Default)]
pub struct TopWordsConfig {
    pub n: usize,
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(counts: WordCount)]
pub struct TopWords {
    config: TopWordsConfig,
}

impl Component for TopWords {
    type Config = TopWordsConfig;
    type Output = Vec<(String, usize)>;

    fn new(config: TopWordsConfig, _: Self::Init<'_>) -> (Self, Self::Output) {
        (TopWords { config }, Vec::new())
    }

    fn update(&mut self, input: Self::Input<'_>) -> Self::Output {
        let mut v: Vec<_> = input.counts.iter().map(|(w, c)| (w.clone(), *c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v.truncate(self.config.n);
        v
    }
}

// --- Longest ---------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(tokens: Tokenize)]
pub struct Longest;

impl Component for Longest {
    type Config = ();
    type Output = Option<String>;

    fn new((): (), _: Self::Init<'_>) -> (Self, Option<String>) {
        (Longest, None)
    }

    fn update(&mut self, input: Self::Input<'_>) -> Option<String> {
        input.tokens.iter().max_by_key(|t| t.len()).cloned()
    }
}

// --- Len<U>: generic over any component that produces Vec<String> ----------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(items: U)]
pub struct Len<U: Node<Output = Vec<String>>>(PhantomData<U>);

impl<U: Node<Output = Vec<String>>> Component for Len<U> {
    type Config = ();
    type Output = usize;

    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        // Generic over the node `U`, `new` can't look at `init.items` (the
        // type depends on `U`'s kind); it starts at 0.
        (Len(PhantomData), 0)
    }

    fn update(&mut self, input: Self::Input<'_>) -> usize {
        input.items.len()
    }
}

// --- Report: three subscriptions --------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(
    top: TopWords,
    longest: Longest,
    line: Line,
)]
pub struct Report;

impl Component for Report {
    type Config = ();
    type Output = String;

    fn new((): (), _: Self::Init<'_>) -> (Self, String) {
        (Report, String::new())
    }

    fn update(&mut self, input: Self::Input<'_>) -> String {
        format!(
            "{:?} -> top {:?}, longest {:?}",
            input.line.0,
            input.top,
            input.longest.as_deref().unwrap_or("-")
        )
    }
}

// --- Printer: the sink. A component whose output is `()` ------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(
    report: Report,
    token_count: Len<Tokenize>,
)]
pub struct Printer {
    pushes: usize,
}

impl Component for Printer {
    type Config = ();
    type Output = ();

    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Printer { pushes: 0 }, ())
    }

    fn update(&mut self, input: Self::Input<'_>) {
        self.pushes += 1;
        println!("\npush {}:", self.pushes);
        println!("  {}", input.report);
        println!("  token count: {}", input.token_count);
    }
}
