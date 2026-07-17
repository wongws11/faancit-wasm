use std::{
    collections::BTreeMap,
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
    const RANK_BLOCK_WORDS: usize = 8;
    println!("cargo::rerun-if-changed={DICTIONARY_PATH}");

    let source = std::fs::read_to_string(DICTIONARY_PATH).expect("failed to read dictionary");
    let dictionary: BTreeMap<String, Pronunciation> =
        serde_json::from_str(&source).expect("failed to parse dictionary");
    let sings = values_by_frequency(&dictionary, |value| value.sing.as_str());
    let wans = values_by_frequency(&dictionary, |value| value.wan.as_str());
    assert!(sings.len() <= 32, "initial index exceeds five bits");
    assert!(wans.len() <= 64, "final index exceeds six bits");

    let mut entries = Vec::with_capacity(dictionary.len());
    for (key, pronunciation) in &dictionary {
        let mut characters = key.chars();
        let character = characters.next().expect("dictionary key must not be empty");
        assert!(
            characters.next().is_none(),
            "dictionary key must be one character"
        );
        assert!(character <= '\u{ffff}', "dictionary character must be BMP");
        let sing = sings
            .iter()
            .position(|value| *value == pronunciation.sing)
            .expect("initial must be indexed");
        let wan = wans
            .iter()
            .position(|value| *value == pronunciation.wan)
            .expect("final must be indexed");
        assert!(
            (1..=6).contains(&pronunciation.tone),
            "tone must be exactly 1 through 6"
        );
        let tone = pronunciation.tone - 1;
        let packed = (sing as u16) | ((wan as u16) << 5) | (u16::from(tone) << 11);
        entries.push((character as u32, packed));
    }

    let minimum = entries.first().expect("dictionary must not be empty").0;
    let maximum = entries.last().expect("dictionary must not be empty").0;
    let word_count = ((maximum - minimum) / 64 + 1) as usize;
    let mut bitmap = vec![0_u64; word_count];
    for &(codepoint, _) in &entries {
        let offset = codepoint - minimum;
        bitmap[offset as usize / 64] |= 1_u64 << (offset % 64);
    }
    let mut ranks = Vec::with_capacity(word_count);
    let mut rank = 0_u16;
    for (index, word) in bitmap.iter().enumerate() {
        if index % RANK_BLOCK_WORDS == 0 {
            ranks.push(rank);
        }
        rank = rank
            .checked_add(word.count_ones() as u16)
            .expect("dictionary rank exceeds u16");
    }
    assert_eq!(usize::from(rank), entries.len());

    let output_path =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set")).join("dictionary.rs");
    let mut output =
        BufWriter::new(File::create(output_path).expect("failed to create dictionary"));

    writeln!(output, "const MIN_CODEPOINT: u32 = {minimum};").unwrap();
    writeln!(output, "const MAX_CODEPOINT: u32 = {maximum};").unwrap();
    writeln!(
        output,
        "const RANK_BLOCK_WORDS: usize = {RANK_BLOCK_WORDS};"
    )
    .unwrap();
    write_table(&mut output, "MEMBERSHIP", "u64", &bitmap);
    write_table(&mut output, "RANKS", "u16", &ranks);
    let pronunciations: Vec<u16> = entries.iter().map(|entry| entry.1).collect();
    write_table(&mut output, "PRONUNCIATIONS", "u16", &pronunciations);
    writeln!(output, "const INITIAL_COUNT: usize = {};", sings.len()).unwrap();
    writeln!(output, "const FINAL_COUNT: usize = {};", wans.len()).unwrap();
    write_ascii_table(&mut output, "INITIAL_ASCII", &sings);
    write_ascii_table(&mut output, "FINAL_ASCII", &wans);
    write_string_table(&mut output, "INITIALS", &sings);
    write_string_table(&mut output, "FINALS", &wans);
    for (name, rust_name) in [
        ("b", "B"),
        ("p", "P"),
        ("d", "D"),
        ("t", "T"),
        ("g", "G"),
        ("k", "K"),
        ("gw", "GW"),
        ("kw", "KW"),
        ("z", "Z"),
        ("c", "C"),
        ("f", "F"),
        ("s", "S"),
        ("h", "H"),
    ] {
        let index = sings
            .iter()
            .position(|value| *value == name)
            .expect("rule initial must exist");
        writeln!(output, "const INITIAL_{rust_name}: u8 = {index};").unwrap();
    }

    writeln!(output, "#[cfg(test)]").unwrap();
    writeln!(output, "static TEST_ENTRIES: &[(u32, u16)] = &[").unwrap();
    for (codepoint, packed) in entries {
        writeln!(output, "    ({codepoint}, {packed}),").unwrap();
    }
    writeln!(output, "];").unwrap();
}

fn values_by_frequency<'a>(
    dictionary: &'a BTreeMap<String, Pronunciation>,
    value: impl Fn(&'a Pronunciation) -> &'a str,
) -> Vec<&'a str> {
    let mut frequencies = BTreeMap::new();
    for pronunciation in dictionary.values() {
        *frequencies.entry(value(pronunciation)).or_insert(0_usize) += 1;
    }
    let mut values: Vec<_> = frequencies.keys().copied().collect();
    values.sort_unstable_by(|left, right| {
        frequencies[right]
            .cmp(&frequencies[left])
            .then_with(|| left.cmp(right))
    });
    values
}

fn write_table<T: std::fmt::Display>(
    output: &mut impl Write,
    name: &str,
    kind: &str,
    values: &[T],
) {
    writeln!(output, "static {name}: &[{kind}] = &[").unwrap();
    for value in values {
        writeln!(output, "    {value},").unwrap();
    }
    writeln!(output, "];").unwrap();
}

fn write_ascii_table(output: &mut impl Write, name: &str, values: &[&str]) {
    let packed: Vec<u32> = values
        .iter()
        .map(|value| {
            assert!(
                value.is_ascii() && value.len() <= 4,
                "syllable part must fit in ASCII u32"
            );
            value
                .bytes()
                .enumerate()
                .fold(0_u32, |word, (shift, byte)| {
                    word | (u32::from(byte) << (shift * 8))
                })
        })
        .collect();
    write_table(output, name, "u32", &packed);
}

fn write_string_table(output: &mut impl Write, name: &str, values: &[&str]) {
    writeln!(output, "#[cfg(not(target_arch = \"wasm32\"))]").unwrap();
    writeln!(output, "static {name}: &[&str] = &[").unwrap();
    for value in values {
        writeln!(output, "    {value:?},").unwrap();
    }
    writeln!(output, "];").unwrap();
}
