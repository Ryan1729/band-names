use std::collections::BTreeSet;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::io::Write;

type Res<A> = Result<A, Box<dyn std::error::Error>>;

struct Words {
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

impl TryFrom<Value> for WordsVecs {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Err("TODO TryFrom<Value> for WordsVecs")
    }
}

impl From<Words> for WordsVecs {
    fn from(words: Words) -> Self {
        todo!()
    }
}

impl From<WordsVecs> for Words {
    fn from(words_vecs: WordsVecs) -> Self {
        todo!()
    }
}

fn main() -> Res<()> {
    // TODO take text as input and add attested adj-noun sequences
    // back to this file
    const PATH: &str = "../../words.js";

    let js_string = std::fs::read_to_string(PATH)?;

    const PREFIX: &str = "words = ";
    let json_str = &js_string[PREFIX.len()..];

    let parsed: Value = serde_json::from_str(json_str)?;

    let words_vecs: WordsVecs = parsed.try_into()?;

    let mut words: Words = words_vecs.into();

    words.attested.insert(("skull".to_string(), "necklace".to_string()));

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

    println!("Wrote successfully");

    Ok(())
}
