use super::*;

fn pronunciation(sing: &'static str, wan: &'static str, tone: i32) -> JyutpingChar {
    JyutpingChar { sing, wan, tone }
}

#[test]
fn looks_up_known_characters() {
    assert_eq!(get_jyutping("德"), Some(pronunciation("d", "ak", 1)));
    assert_eq!(get_jyutping("紅"), Some(pronunciation("h", "ung", 4)));
    assert_eq!(get_jyutping("不存在的字zz"), None);
}

#[test]
fn formats_pronunciation() {
    assert_eq!(pronunciation("d", "ak", 1).to_string(), "d ak 1");
    assert_eq!(pronunciation("", "ng", 4).to_string(), " ng 4");
}

#[test]
fn classifies_initials_and_tones() {
    for sing in ["b", "p", "d", "t", "g", "k", "gw", "kw", "z", "c"] {
        assert!(is_stop(&pronunciation(sing, "", 0)));
    }
    for sing in ["f", "s", "h", "m", "n", "ng", "l", "j", "w", ""] {
        assert!(!is_stop(&pronunciation(sing, "", 0)));
    }
    for sing in ["f", "s", "h"] {
        assert!(is_fricative(&pronunciation(sing, "", 0)));
    }
    for tone in 1..=6 {
        assert_eq!(
            is_ping(&pronunciation("", "", tone)),
            tone == 1 || tone == 4
        );
        assert_eq!(is_yam(&pronunciation("", "", tone)), tone <= 3);
    }
}

#[test]
fn combines_upper_initial_with_lower_final() {
    let result = get_faancit(&pronunciation("d", "ak", 1), &pronunciation("g", "ung", 4));

    assert_eq!(result.pronunciation, pronunciation("d", "ung", 1));
    assert!(result.homophones.contains(&'冬'));
}

#[test]
fn applies_aspiration_and_deaspiration() {
    let aspirated = get_faancit(&pronunciation("b", "ei", 4), &pronunciation("g", "ung", 4));
    assert_eq!(aspirated.pronunciation.sing, "p");
    assert!(aspirated.diagnostics.contains("下字平聲則聲母送氣"));

    let deaspirated = get_faancit(&pronunciation("p", "ei", 4), &pronunciation("g", "ung", 6));
    assert_eq!(deaspirated.pronunciation.sing, "b");
    assert!(deaspirated.diagnostics.contains("下字仄聲則聲母不送氣"));
}

#[test]
fn applies_labiodental_shift() {
    let result = get_faancit(&pronunciation("f", "ong", 1), &pronunciation("g", "ung", 1));

    assert_eq!(result.pronunciation.sing, "b");
    assert_eq!(result.diagnostics, "古無輕唇音\n");
}

#[test]
fn maps_yin_and_yang_tones() {
    for (lower_tone, expected) in [(4, 1), (5, 2), (6, 3)] {
        let result = get_faancit(
            &pronunciation("l", "ai", 1),
            &pronunciation("l", "ung", lower_tone),
        );
        assert_eq!(result.pronunciation.tone, expected);
    }
    for (lower_tone, expected) in [(1, 4), (2, 5), (3, 6)] {
        let result = get_faancit(
            &pronunciation("l", "ai", 4),
            &pronunciation("l", "ung", lower_tone),
        );
        assert_eq!(result.pronunciation.tone, expected);
    }
}

#[test]
fn applies_yang_tone_adjustment_after_mapping() {
    let stop = get_faancit(&pronunciation("b", "ai", 4), &pronunciation("l", "ung", 2));
    assert_eq!(stop.pronunciation.tone, 6);

    let fricative = get_faancit(&pronunciation("l", "ai", 4), &pronunciation("f", "ung", 2));
    assert_eq!(fricative.pronunciation.tone, 6);
}

#[test]
fn generated_dictionary_is_complete_and_compact() {
    assert_eq!(ENTRIES.len(), 6_059);
    assert_eq!(SINGS.len(), 20);
    assert_eq!(WANS.len(), 53);
    assert_eq!(size_of::<DictionaryEntry>(), 8);
    assert!(
        ENTRIES
            .windows(2)
            .all(|pair| pair[0].character < pair[1].character)
    );
    assert!(ENTRIES.iter().all(|entry| (1..=6).contains(&entry.tone)));

    let result = get_faancit(&pronunciation("d", "ak", 1), &pronunciation("g", "ung", 4));
    assert!(result.homophones.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(result.homophones.contains(&'冬'));
    assert!(result.homophones.contains(&'東'));
}
