use std::collections::BTreeMap;
use std::marker::PhantomData;

use dataflow_flat::{Component, ComponentSubscription};

// --- Source: subscribes to nothing, emits one line per run -----------------

pub struct SourceConfig {
    pub lines: Vec<String>,
}

// Invented default: a single empty line, so `update` never divides by zero.
impl Default for SourceConfig {
    fn default() -> Self {
        SourceConfig { lines: vec![String::new()] }
    }
}

#[derive(ComponentSubscription)]
pub struct Source {
    config: SourceConfig,
    next: usize,
}

impl Component for Source {
    type Config = SourceConfig;
    type Output = String;

    fn new(config: SourceConfig) -> Self {
        Source { config, next: 0 }
    }

    fn update(&mut self, _: Self::Input<'_>) -> String {
        let line = self.config.lines[self.next % self.config.lines.len()].clone();
        self.next += 1;
        line
    }
}

// --- Tokenize: three components subscribe to it; it still runs once -------

#[derive(Default)]
pub struct TokenizeConfig {
    pub lowercase: bool,
}

#[derive(ComponentSubscription)]
#[subscribe(line: Source)]
pub struct Tokenize {
    config: TokenizeConfig,
}

impl Component for Tokenize {
    type Config = TokenizeConfig;
    type Output = Vec<String>;

    fn new(config: TokenizeConfig) -> Self {
        Tokenize { config }
    }

    fn update(&mut self, input: Self::Input<'_>) -> Vec<String> {
        println!("    (Tokenize::update)");
        input
            .line
            .split_whitespace()
            .map(|w| if self.config.lowercase { w.to_lowercase() } else { w.to_string() })
            .collect()
    }
}

// --- WordCount: unit config (`()` is `Default`), so it needs no entry in Configs

#[derive(ComponentSubscription)]
#[subscribe(tokens: Tokenize)]
pub struct WordCount;

impl Component for WordCount {
    type Config = ();
    type Output = BTreeMap<String, usize>;

    fn new((): ()) -> Self {
        WordCount
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

#[derive(ComponentSubscription)]
#[subscribe(counts: WordCount)]
pub struct TopWords {
    config: TopWordsConfig,
}

impl Component for TopWords {
    type Config = TopWordsConfig;
    type Output = Vec<(String, usize)>;

    fn new(config: TopWordsConfig) -> Self {
        TopWords { config }
    }

    fn update(&mut self, input: Self::Input<'_>) -> Self::Output {
        let mut v: Vec<_> = input.counts.iter().map(|(w, c)| (w.clone(), *c)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v.truncate(self.config.n);
        v
    }
}

// --- Longest ---------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(tokens: Tokenize)]
pub struct Longest;

impl Component for Longest {
    type Config = ();
    type Output = Option<String>;

    fn new((): ()) -> Self {
        Longest
    }

    fn update(&mut self, input: Self::Input<'_>) -> Option<String> {
        input.tokens.iter().max_by_key(|t| t.len()).cloned()
    }
}

// --- Len<U>: generic over any component that produces Vec<String> ----------

#[derive(ComponentSubscription)]
#[subscribe(items: U)]
pub struct Len<U: Component<Output = Vec<String>>>(PhantomData<U>);

impl<U: Component<Output = Vec<String>>> Component for Len<U> {
    type Config = ();
    type Output = usize;

    fn new((): ()) -> Self {
        Len(PhantomData)
    }

    fn update(&mut self, input: Self::Input<'_>) -> usize {
        input.items.len()
    }
}

// --- Report: three subscriptions --------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(
    top: TopWords,
    longest: Longest,
    line: Source,
)]
pub struct Report;

impl Component for Report {
    type Config = ();
    type Output = String;

    fn new((): ()) -> Self {
        Report
    }

    fn update(&mut self, input: Self::Input<'_>) -> String {
        format!(
            "{:?} -> top {:?}, longest {:?}",
            input.line,
            input.top,
            input.longest.as_deref().unwrap_or("-")
        )
    }
}
