use std::collections::{BTreeSet, HashSet};
use serde::{Serialize, Deserialize};
use std::io::Write;

type Res<A> = Result<A, Box<dyn std::error::Error>>;

// Let's keep the attested values in a consistent order.
type AttestedSet = BTreeSet<(String, String)>;

struct Words {
    attested: AttestedSet,
    adjectives: Vec<String>,
    nouns: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct WordsVecs {
    attested: Vec<(String, String)>,
    adjectives: Vec<String>,
    nouns: Vec<String>,
}

impl From<Words> for WordsVecs {
    fn from(words: Words) -> Self {
        WordsVecs {
            attested: words.attested.into_iter().collect(),
            adjectives: words.adjectives,
            nouns: words.nouns,
        }
    }
}

impl From<WordsVecs> for Words {
    fn from(words_vecs: WordsVecs) -> Self {
        Words {
            attested: BTreeSet::from_iter(words_vecs.attested.into_iter()),
            adjectives: words_vecs.adjectives,
            nouns: words_vecs.nouns,
        }
    }
}

fn main() -> Res<()> {
    const PATH: &str = "../../words.js";

    enum Mode {
        WriteBack,
        PrintFound,
        PrintAdditional,
    }

    let mode = {
        let mut mode = Mode::WriteBack;

        let mut seen_exe_name = false;
        for arg in std::env::args() {
            match arg.as_str() {
                "--write-back" => {
                    mode = Mode::WriteBack;
                }
                "--print-found" => {
                    mode = Mode::PrintFound;
                }
                "--print-additional" => {
                    mode = Mode::PrintAdditional;
                }
                other => {
                    if other.trim() != "" {
                        if seen_exe_name {
                            return Err(format!("Unknown arg: \"{arg}\"").into());
                        }
                        seen_exe_name = true;
                    }
                }
            }
        }

        mode
    };

    let js_string = std::fs::read_to_string(PATH)?;

    const PREFIX: &str = "words = ";
    let json_str = &js_string[PREFIX.len()..];

    let words_vecs: WordsVecs = serde_json::from_str(json_str)?;

    let mut words: Words = words_vecs.into();

    let adjectives: HashSet<String> = HashSet::from_iter(words.adjectives.clone());
    let nouns: HashSet<String> = HashSet::from_iter(words.nouns.clone());

    let lines = std::io::stdin().lines().filter_map(|res| res.ok());

    match mode {
        Mode::WriteBack => {
            let pre_count = words.attested.len();

            add_pairs(&mut words.attested, lines, &adjectives, &nouns)?;

            let post_count = words.attested.len();

            if post_count <= pre_count {
                println!("No new adjective-noun pairs found.");

                return Ok(());
            }

            let words_vecs: WordsVecs = words.into();

            let mut output = std::io::Cursor::new(Vec::with_capacity(1 << 12));
            write!(&mut output, "{PREFIX}")?;

            serde_json::ser::to_writer_pretty(
                &mut output,
                &words_vecs
            )?;

            std::fs::write(
                PATH,
                output.get_ref()
            )?;

            let additional = post_count - pre_count;
            println!(
                "Successfully wrote {additional} more adjective-noun pair{}.",
                if additional == 1 { "" } else { "s" }
            );
        },
        Mode::PrintAdditional | Mode::PrintFound => {
            let mut attested = AttestedSet::new();

            add_pairs(&mut attested, lines, &adjectives, &nouns)?;

            if let Mode::PrintAdditional = mode {
                attested.retain(|k| {
                    !words.attested.contains(k)
                });
            }

            if attested.is_empty() {
                println!("No new adjective-noun pairs found.");

                return Ok(());
            }

            for (adj, noun) in attested {
                println!("{adj} {noun}");
            }
        },
    }

    Ok(())
}

fn add_pairs(
    attested: &mut AttestedSet,
    // TODO Are we missing pairs that are split across lines?
    lines: impl Iterator<Item = String>,
    adjectives: &HashSet<String>,
    nouns: &HashSet<String>,
) -> Res<()> {
    let pattern = |c: char|
        c.is_whitespace()
        || c == '.'
        || c == '!'
        || c == '?'
        || c == '"'
        || c == ':'
        || c == ';'
        ;

    let mut adj_lower = String::with_capacity(32);
    let mut noun_lower = String::with_capacity(32);

    let mut lines = lines.peekable();

    while let Some(line) = lines.next() {
        let mut iter = line.split(pattern).peekable();
        while let Some(poss_adj) = iter.next() {
            let peeked = match iter.peek() {
                Some(poss_noun) => Some(poss_noun.to_string()),
                None => lines.peek()
                    .and_then(|line|
                        line.split(pattern)
                            .next()
                            .map(|poss_noun| poss_noun.to_string())
                    ),
            };
            if let Some(poss_noun) = peeked {
                use std::fmt::Write;
                if (
                    // First so adj_lower is always up to date when we
                    // get a match
                    {
                        adj_lower.clear();

                        write!(&mut adj_lower, "{poss_adj}")?;
                        adj_lower.make_ascii_lowercase();

                        adjectives.contains(&adj_lower)
                    }
                    || adjectives.contains(poss_adj)
                )
                && (
                    // First so noun_lower is always up to date when we
                    // get a match
                    {
                        noun_lower.clear();

                        write!(&mut noun_lower, "{poss_noun}")?;
                        noun_lower.make_ascii_lowercase();

                        nouns.contains(&noun_lower)
                    }
                    || nouns.contains(&poss_noun)
                ) {
                    attested.insert((
                        adj_lower.clone(),
                        noun_lower.clone()
                    ));
                }
            }
        }
    }

    Ok(())
}

#[test]
fn add_pairs_works_on_this_singleline_example() {
    let mut attested = AttestedSet::new();

    const ADJ: &str = "adj";
    const NOUN: &str = "noun";
    macro_rules! adj {
        () => {
            ADJ.to_string()
        }
    }
    macro_rules! noun {
        () => {
            NOUN.to_string()
        }
    }

    add_pairs(
        &mut attested,
        [format!("{} {}", adj!(), noun!())].into_iter(),
        &HashSet::from([adj!()]),
        &HashSet::from([noun!()]),
    ).unwrap();

    assert!(attested.contains(&(adj!(), noun!())));
}

#[test]
fn add_pairs_works_on_this_multiline_example() {
    let mut attested = AttestedSet::new();

    const ADJ: &str = "adj";
    const NOUN: &str = "noun";
    macro_rules! adj {
        () => {
            ADJ.to_string()
        }
    }
    macro_rules! noun {
        () => {
            NOUN.to_string()
        }
    }

    add_pairs(
        &mut attested,
        [adj!(), noun!()].into_iter(),
        &HashSet::from([adj!()]),
        &HashSet::from([noun!()]),
    ).unwrap();

    assert!(attested.contains(&(adj!(), noun!())));
}

// TODO? Add test for words hypenated only to split across lines and make it
// pass? Or is it better to just preprocess the input if that actually
// comes up a lot?

// TODO It would be nice to have phrases like "warm root beer", that contain
// nested noun phraes consisting of adjective noun pairs, preceeded by
// another adjective be included as well, given they are attested. So we
// could use the existing set of attested pairs to do that.

// TODO We end up with a lot of words that appear in multiple attested pairs
// It seems like filtering out pairs that contain words that show up in
// multiple pairs would produce more interesting results on average.