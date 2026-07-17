use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JyutpingChar {
    pub sing: &'static str,
    pub wan: &'static str,
    pub tone: i32,
}

impl fmt::Display for JyutpingChar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.sing, self.wan, self.tone)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Faancit {
    pub pronunciation: JyutpingChar,
    pub homophones: Vec<char>,
    pub diagnostics: String,
}

#[repr(C)]
struct DictionaryEntry {
    character: char,
    sing: u8,
    wan: u8,
    tone: u8,
}

include!(concat!(env!("OUT_DIR"), "/dictionary.rs"));

impl DictionaryEntry {
    fn pronunciation(&self) -> JyutpingChar {
        JyutpingChar {
            sing: SINGS[self.sing as usize],
            wan: WANS[self.wan as usize],
            tone: i32::from(self.tone),
        }
    }
}

pub fn get_jyutping(character: &str) -> Option<JyutpingChar> {
    let mut characters = character.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }

    ENTRIES
        .binary_search_by_key(&character, |entry| entry.character)
        .ok()
        .map(|index| ENTRIES[index].pronunciation())
}

pub fn get_faancit(upper: &JyutpingChar, lower: &JyutpingChar) -> Faancit {
    let mut pronunciation = JyutpingChar {
        sing: upper.sing,
        wan: lower.wan,
        tone: lower.tone,
    };
    let mut diagnostics = String::new();

    if !is_yam(upper) && is_stop(upper) {
        diagnostics.push_str("上字陽聲塞音\n");
        if is_ping(lower) {
            diagnostics.push_str("下字平聲則聲母送氣\n");
            pronunciation.sing = match upper.sing {
                "b" => "p",
                "d" => "t",
                "g" => "k",
                "gw" => "kw",
                "z" => "c",
                sing => sing,
            };
        } else {
            diagnostics.push_str("下字仄聲則聲母不送氣\n");
            pronunciation.sing = match upper.sing {
                "p" => "b",
                "t" => "d",
                "k" => "g",
                "kw" => "gw",
                "c" => "z",
                sing => sing,
            };
        }
    }

    if upper.sing == "f" {
        diagnostics.push_str("古無輕唇音\n");
        pronunciation.sing = "b";
    }

    if is_yam(upper) && !is_yam(lower) {
        diagnostics.push_str("上字陰下字陽\n");
        pronunciation.tone = match lower.tone {
            4 => 1,
            5 => 2,
            6 => 3,
            tone => tone,
        };
    } else if !is_yam(upper) && is_yam(lower) {
        diagnostics.push_str("上字陽下字陰\n");
        pronunciation.tone = match lower.tone {
            1 => 4,
            2 => 5,
            3 => 6,
            tone => tone,
        };
    }

    if !is_yam(upper) && (is_stop(upper) || is_fricative(lower)) {
        diagnostics.push_str("上字陽聲而上字塞音或下字擦音\n");
        pronunciation.tone = match pronunciation.tone {
            2 => 3,
            5 => 6,
            tone => tone,
        };
    }

    let homophones = ENTRIES
        .iter()
        .filter(|entry| entry.pronunciation() == pronunciation)
        .map(|entry| entry.character)
        .collect();

    Faancit {
        pronunciation,
        homophones,
        diagnostics,
    }
}

fn is_stop(character: &JyutpingChar) -> bool {
    matches!(
        character.sing,
        "b" | "p" | "d" | "t" | "g" | "k" | "gw" | "kw" | "z" | "c"
    )
}

fn is_fricative(character: &JyutpingChar) -> bool {
    matches!(character.sing, "f" | "s" | "h")
}

fn is_ping(character: &JyutpingChar) -> bool {
    matches!(character.tone, 1 | 4)
}

fn is_yam(character: &JyutpingChar) -> bool {
    character.tone <= 3
}

#[cfg(test)]
mod tests;
