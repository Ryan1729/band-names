use std::collections::{BTreeSet, HashSet};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::io::Write;

type Res<A> = Result<A, Box<dyn std::error::Error>>;

struct Words {
    // Let's keep the attested values in a consistent order.
    attested: BTreeSet<(String, String)>,
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

    let js_string = std::fs::read_to_string(PATH)?;

    const PREFIX: &str = "words = ";
    let json_str = &js_string[PREFIX.len()..];

    let words_vecs: WordsVecs = serde_json::from_str(json_str)?;

    let mut words: Words = words_vecs.into();

    let adjectives: HashSet<String> = HashSet::from_iter(words.adjectives.clone());
    let nouns: HashSet<String> = HashSet::from_iter(words.nouns.clone());

    let pre_count = words.attested.len();

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

    for line in std::io::stdin().lines() {
        let line = line?;

        let mut iter = line.split(pattern).peekable();
        while let Some(poss_adj) = iter.next() {
            if let Some(poss_noun) = iter.peek() {
                let poss_noun: &str = poss_noun;

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
                    || nouns.contains(poss_noun)
                ) {
                    words.attested.insert((
                        adj_lower.clone(),
                        noun_lower.clone()
                    ));
                }
            }
        }
    }

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

    Ok(())
}