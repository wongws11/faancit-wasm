use std::fmt;

const INITIAL_BITS: u32 = 5;
const FINAL_BITS: u32 = 6;
const INITIAL_MASK: u16 = (1 << INITIAL_BITS) - 1;
const FINAL_MASK: u16 = (1 << FINAL_BITS) - 1;
const TONE_SHIFT: u32 = INITIAL_BITS + FINAL_BITS;

pub(crate) const DIAG_UPPER_YANG_STOP: u16 = 1 << 0;
pub(crate) const DIAG_ASPIRATED: u16 = 1 << 1;
pub(crate) const DIAG_DEASPIRATED: u16 = 1 << 2;
pub(crate) const DIAG_LABIODENTAL: u16 = 1 << 3;
pub(crate) const DIAG_YIN_OVER_YANG: u16 = 1 << 4;
pub(crate) const DIAG_YANG_OVER_YIN: u16 = 1 << 5;
pub(crate) const DIAG_YANG_ADJUSTMENT: u16 = 1 << 6;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Tone {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
}

impl Tone {
    const fn index(self) -> u16 {
        self as u16 - 1
    }
}

impl TryFrom<u8> for Tone {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Tone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", *self as u8)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JyutpingChar(u16);

impl JyutpingChar {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(initial: &str, final_: &str, tone: Tone) -> Option<Self> {
        let initial = INITIALS
            .iter()
            .position(|candidate| *candidate == initial)? as u8;
        let final_ = FINALS.iter().position(|candidate| *candidate == final_)? as u8;
        Self::from_parts(initial, final_, tone)
    }

    pub fn from_parts(initial: u8, final_: u8, tone: Tone) -> Option<Self> {
        if usize::from(initial) >= INITIAL_COUNT || usize::from(final_) >= FINAL_COUNT {
            return None;
        }
        Some(Self(
            u16::from(initial) | (u16::from(final_) << INITIAL_BITS) | (tone.index() << TONE_SHIFT),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn initial(&self) -> &'static str {
        INITIALS[self.initial_index() as usize]
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn final_(&self) -> &'static str {
        FINALS[self.final_index() as usize]
    }

    pub const fn initial_index(self) -> u8 {
        (self.0 & INITIAL_MASK) as u8
    }

    pub const fn final_index(self) -> u8 {
        ((self.0 >> INITIAL_BITS) & FINAL_MASK) as u8
    }

    pub fn tone(self) -> Tone {
        // Construction and raw decoding validate this three-bit field.
        Tone::try_from(((self.0 >> TONE_SHIFT) + 1) as u8).unwrap_or(Tone::One)
    }

    pub const fn packed(self) -> u16 {
        self.0
    }

    pub(crate) fn from_packed(packed: u32) -> Option<Self> {
        let packed = u16::try_from(packed).ok()?;
        let initial = (packed & INITIAL_MASK) as usize;
        let final_ = ((packed >> INITIAL_BITS) & FINAL_MASK) as usize;
        let tone = packed >> TONE_SHIFT;
        if initial >= INITIAL_ASCII.len()
            || final_ >= FINAL_ASCII.len()
            || tone >= 6
            || packed >> (TONE_SHIFT + 3) != 0
        {
            return None;
        }
        Some(Self(packed))
    }
}

impl fmt::Display for JyutpingChar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_ascii(INITIAL_ASCII[self.initial_index() as usize], formatter)?;
        formatter.write_str(" ")?;
        format_ascii(FINAL_ASCII[self.final_index() as usize], formatter)?;
        write!(formatter, " {}", self.tone())
    }
}

fn format_ascii(mut packed: u32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    while packed != 0 {
        write!(formatter, "{}", (packed as u8) as char)?;
        packed >>= 8;
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Faancit {
    pub pronunciation: JyutpingChar,
    pub homophones: Vec<char>,
    pub diagnostics: String,
}

include!(concat!(env!("OUT_DIR"), "/dictionary.rs"));

pub fn get_jyutping(character: &str) -> Option<JyutpingChar> {
    let mut characters = character.chars();
    let character = characters.next()?;
    if characters.next().is_some() {
        return None;
    }
    lookup_codepoint(character as u32)
}

pub fn get_faancit(upper: &JyutpingChar, lower: &JyutpingChar) -> Faancit {
    let (pronunciation, diagnostics) = combine_packed(upper.0, lower.0);
    let homophones = homophones(pronunciation).collect();
    let mut text = String::new();
    append_diagnostics(diagnostics, &mut text);
    Faancit {
        pronunciation: JyutpingChar(pronunciation),
        homophones,
        diagnostics: text,
    }
}

pub(crate) fn lookup_codepoint(codepoint: u32) -> Option<JyutpingChar> {
    let offset = codepoint.checked_sub(MIN_CODEPOINT)?;
    if codepoint > MAX_CODEPOINT {
        return None;
    }
    let word_index = offset as usize / 64;
    let bit_index = offset % 64;
    let word = MEMBERSHIP[word_index];
    let bit = 1_u64 << bit_index;
    if word & bit == 0 {
        return None;
    }
    let lower_bits = if bit_index == 0 { 0 } else { bit - 1 };
    let block_index = word_index / RANK_BLOCK_WORDS;
    let block_start = block_index * RANK_BLOCK_WORDS;
    let preceding_words: usize = MEMBERSHIP[block_start..word_index]
        .iter()
        .map(|word| word.count_ones() as usize)
        .sum();
    let rank = usize::from(RANKS[block_index])
        + preceding_words
        + (word & lower_bits).count_ones() as usize;
    JyutpingChar::from_packed(u32::from(PRONUNCIATIONS[rank]))
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn combine_raw(upper: u32, lower: u32) -> Option<(u16, u16)> {
    let upper = JyutpingChar::from_packed(upper)?;
    let lower = JyutpingChar::from_packed(lower)?;
    Some(combine_packed(upper.0, lower.0))
}

fn combine_packed(upper: u16, lower: u16) -> (u16, u16) {
    let upper_initial = (upper & INITIAL_MASK) as u8;
    let lower_initial = (lower & INITIAL_MASK) as u8;
    let mut initial = upper_initial;
    let final_ = lower & (FINAL_MASK << INITIAL_BITS);
    let upper_tone = ((upper >> TONE_SHIFT) + 1) as u8;
    let lower_tone = ((lower >> TONE_SHIFT) + 1) as u8;
    let mut tone = lower_tone;
    let mut diagnostics = 0;

    if !is_yin(upper_tone) && is_stop(upper_initial) {
        diagnostics |= DIAG_UPPER_YANG_STOP;
        if is_level(lower_tone) {
            diagnostics |= DIAG_ASPIRATED;
            initial = match upper_initial {
                INITIAL_B => INITIAL_P,
                INITIAL_D => INITIAL_T,
                INITIAL_G => INITIAL_K,
                INITIAL_GW => INITIAL_KW,
                INITIAL_Z => INITIAL_C,
                value => value,
            };
        } else {
            diagnostics |= DIAG_DEASPIRATED;
            initial = match upper_initial {
                INITIAL_P => INITIAL_B,
                INITIAL_T => INITIAL_D,
                INITIAL_K => INITIAL_G,
                INITIAL_KW => INITIAL_GW,
                INITIAL_C => INITIAL_Z,
                value => value,
            };
        }
    }

    if upper_initial == INITIAL_F {
        diagnostics |= DIAG_LABIODENTAL;
        initial = INITIAL_B;
    }

    if is_yin(upper_tone) && !is_yin(lower_tone) {
        diagnostics |= DIAG_YIN_OVER_YANG;
        tone -= 3;
    } else if !is_yin(upper_tone) && is_yin(lower_tone) {
        diagnostics |= DIAG_YANG_OVER_YIN;
        tone += 3;
    }

    if !is_yin(upper_tone) && (is_stop(upper_initial) || is_fricative(lower_initial)) {
        diagnostics |= DIAG_YANG_ADJUSTMENT;
        if tone == 2 || tone == 5 {
            tone += 1;
        }
    }

    let packed = u16::from(initial) | final_ | (u16::from(tone - 1) << TONE_SHIFT);
    (packed, diagnostics)
}

#[cfg(any(test, target_arch = "wasm32"))]
pub(crate) fn next_homophone(packed: u32, after_codepoint: u32) -> Option<u32> {
    let packed = JyutpingChar::from_packed(packed)?.0;
    let start = after_codepoint.saturating_add(1).max(MIN_CODEPOINT);
    if start > MAX_CODEPOINT {
        return None;
    }
    (start..=MAX_CODEPOINT)
        .find(|codepoint| lookup_codepoint(*codepoint).is_some_and(|value| value.0 == packed))
}

fn homophones(packed: u16) -> impl Iterator<Item = char> {
    (MIN_CODEPOINT..=MAX_CODEPOINT).filter_map(move |codepoint| {
        lookup_codepoint(codepoint)
            .filter(|candidate| candidate.0 == packed)
            .and_then(|_| char::from_u32(codepoint))
    })
}

fn append_diagnostics(flags: u16, output: &mut String) {
    for (flag, text) in [
        (DIAG_UPPER_YANG_STOP, "上字陽聲塞音\n"),
        (DIAG_ASPIRATED, "下字平聲則聲母送氣\n"),
        (DIAG_DEASPIRATED, "下字仄聲則聲母不送氣\n"),
        (DIAG_LABIODENTAL, "古無輕唇音\n"),
        (DIAG_YIN_OVER_YANG, "上字陰下字陽\n"),
        (DIAG_YANG_OVER_YIN, "上字陽下字陰\n"),
        (DIAG_YANG_ADJUSTMENT, "上字陽聲而上字塞音或下字擦音\n"),
    ] {
        if flags & flag != 0 {
            output.push_str(text);
        }
    }
}

fn is_stop(initial: u8) -> bool {
    matches!(
        initial,
        INITIAL_B
            | INITIAL_P
            | INITIAL_D
            | INITIAL_T
            | INITIAL_G
            | INITIAL_K
            | INITIAL_GW
            | INITIAL_KW
            | INITIAL_Z
            | INITIAL_C
    )
}

fn is_fricative(initial: u8) -> bool {
    matches!(initial, INITIAL_F | INITIAL_S | INITIAL_H)
}

const fn is_level(tone: u8) -> bool {
    tone == 1 || tone == 4
}

const fn is_yin(tone: u8) -> bool {
    tone <= 3
}

#[cfg(test)]
mod tests;
