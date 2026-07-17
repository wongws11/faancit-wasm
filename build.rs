use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde::Deserialize;

#[derive(Deserialize)]
struct Pronunciation {
    sing: String,
    wan: String,
    tone: u8,
}

fn main() {
    const DICTIONARY_PATH: &str = "jyutping/jyutping.json";
    println!("cargo::rerun-if-changed={DICTIONARY_PATH}");

    let source = std::fs::read_to_string(DICTIONARY_PATH).expect("failed to read dictionary");
    let dictionary: BTreeMap<String, Pronunciation> =
        serde_json::from_str(&source).expect("failed to parse dictionary");
    let sings: Vec<&str> = dictionary
        .values()
        .map(|value| value.sing.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let wans: Vec<&str> = dictionary
        .values()
        .map(|value| value.wan.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    assert!(sings.len() <= u8::MAX as usize);
    assert!(wans.len() <= u8::MAX as usize);

    let output_path =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set")).join("dictionary.rs");
    let mut output =
        BufWriter::new(File::create(output_path).expect("failed to create dictionary"));

    write_string_table(&mut output, "SINGS", &sings);
    write_string_table(&mut output, "WANS", &wans);
    writeln!(output, "static ENTRIES: &[DictionaryEntry] = &[")
        .expect("failed to write dictionary");

    for (character, pronunciation) in &dictionary {
        let mut characters = character.chars();
        let character = characters.next().expect("dictionary key must not be empty");
        assert!(
            characters.next().is_none(),
            "dictionary key must be one character"
        );
        let sing = sings
            .binary_search(&pronunciation.sing.as_str())
            .expect("initial must be indexed");
        let wan = wans
            .binary_search(&pronunciation.wan.as_str())
            .expect("final must be indexed");

        writeln!(
            output,
            "    DictionaryEntry {{ character: {character:?}, sing: {sing}, wan: {wan}, tone: {} }},",
            pronunciation.tone
        )
        .expect("failed to write dictionary");
    }

    writeln!(output, "];").expect("failed to finish dictionary");
}

fn write_string_table(output: &mut impl Write, name: &str, values: &[&str]) {
    writeln!(output, "static {name}: &[&str] = &[").expect("failed to write string table");
    for value in values {
        writeln!(output, "    {value:?},").expect("failed to write string table");
    }
    writeln!(output, "];").expect("failed to finish string table");
}
