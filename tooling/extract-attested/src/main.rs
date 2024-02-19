type Res<A> = Result<A, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    const PATH: &str = "../../words.js";

    // TODO take text as input and add attested adj-noun sequences
    // back to this file
    let s = std::fs::read_to_string(PATH)?;

    println!("{s}");

    std::fs::write(PATH, s)?;

    Ok(())
}
