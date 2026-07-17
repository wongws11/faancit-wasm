use super::*;

fn pronunciation(initial: &str, final_: &str, tone: Tone) -> JyutpingChar {
    JyutpingChar::new(initial, final_, tone).unwrap()
}

#[test]
fn exhaustively_looks_up_and_round_trips_dictionary() {
    assert_eq!(TEST_ENTRIES.len(), 6_059);
    for &(codepoint, packed) in TEST_ENTRIES {
        let expected = JyutpingChar::from_packed(u32::from(packed)).unwrap();
        assert_eq!(lookup_codepoint(codepoint), Some(expected));
        assert_eq!(expected.packed(), packed);
        assert_eq!(
            JyutpingChar::new(expected.initial(), expected.final_(), expected.tone()),
            Some(expected)
        );
    }
    assert_eq!(
        get_jyutping("德"),
        Some(pronunciation("d", "ak", Tone::One))
    );
    assert_eq!(
        get_jyutping("紅"),
        Some(pronunciation("h", "ung", Tone::Four))
    );
    assert_eq!(get_jyutping("不存在的字zz"), None);
}

#[test]
fn bitmap_and_rank_are_exact() {
    assert_eq!(RANKS.len(), MEMBERSHIP.len().div_ceil(RANK_BLOCK_WORDS));
    let mut count = 0_u16;
    for (index, word) in MEMBERSHIP.iter().copied().enumerate() {
        if index % RANK_BLOCK_WORDS == 0 {
            assert_eq!(RANKS[index / RANK_BLOCK_WORDS], count);
        }
        count += word.count_ones() as u16;
    }
    assert_eq!(usize::from(count), TEST_ENTRIES.len());
    assert_eq!(TEST_ENTRIES.first().unwrap().0, MIN_CODEPOINT);
    assert_eq!(TEST_ENTRIES.last().unwrap().0, MAX_CODEPOINT);
    for codepoint in MIN_CODEPOINT..=MAX_CODEPOINT {
        assert_eq!(
            lookup_codepoint(codepoint).is_some(),
            TEST_ENTRIES
                .binary_search_by_key(&codepoint, |entry| entry.0)
                .is_ok()
        );
    }
}

#[test]
fn constructors_and_raw_helpers_reject_invalid_values() {
    assert_eq!(Tone::try_from(0), Err(()));
    assert_eq!(Tone::try_from(7), Err(()));
    assert!(JyutpingChar::new("nope", "ak", Tone::One).is_none());
    assert!(JyutpingChar::new("d", "nope", Tone::One).is_none());
    assert!(JyutpingChar::from_parts(INITIALS.len() as u8, 0, Tone::One).is_none());
    assert!(JyutpingChar::from_parts(0, FINALS.len() as u8, Tone::One).is_none());
    for packed in [u32::MAX, 1 << 16, 6 << TONE_SHIFT, 0x8000] {
        assert!(JyutpingChar::from_packed(packed).is_none());
        assert_eq!(combine_raw(packed, 0), None);
        assert_eq!(next_homophone(packed, 0), None);
    }
}

#[test]
fn formats_public_values() {
    assert_eq!(Tone::Six.to_string(), "6");
    assert_eq!(pronunciation("d", "ak", Tone::One).to_string(), "d ak 1");
    assert_eq!(pronunciation("", "ng", Tone::Four).to_string(), " ng 4");
}

#[test]
fn applies_aspiration_and_deaspiration_rules() {
    for (source, expected) in [("b", "p"), ("d", "t"), ("g", "k"), ("gw", "kw"), ("z", "c")] {
        let result = get_faancit(
            &pronunciation(source, "ai", Tone::Four),
            &pronunciation("l", "ung", Tone::Four),
        );
        assert_eq!(result.pronunciation.initial(), expected);
        assert_eq!(
            result.diagnostics,
            "上字陽聲塞音\n下字平聲則聲母送氣\n上字陽聲而上字塞音或下字擦音\n"
        );
    }
    for (source, expected) in [("p", "b"), ("t", "d"), ("k", "g"), ("kw", "gw"), ("c", "z")] {
        let result = get_faancit(
            &pronunciation(source, "ai", Tone::Four),
            &pronunciation("l", "ung", Tone::Six),
        );
        assert_eq!(result.pronunciation.initial(), expected);
        assert_eq!(
            result.diagnostics,
            "上字陽聲塞音\n下字仄聲則聲母不送氣\n上字陽聲而上字塞音或下字擦音\n"
        );
    }
}

#[test]
fn applies_remaining_rules_and_exact_diagnostics() {
    let labiodental = get_faancit(
        &pronunciation("f", "ong", Tone::One),
        &pronunciation("g", "ung", Tone::One),
    );
    assert_eq!(labiodental.pronunciation.initial(), "b");
    assert_eq!(labiodental.diagnostics, "古無輕唇音\n");

    for (upper, lower, expected, diagnostic) in [
        (Tone::One, Tone::Four, Tone::One, "上字陰下字陽\n"),
        (Tone::One, Tone::Five, Tone::Two, "上字陰下字陽\n"),
        (Tone::One, Tone::Six, Tone::Three, "上字陰下字陽\n"),
        (Tone::Four, Tone::One, Tone::Four, "上字陽下字陰\n"),
        (Tone::Four, Tone::Two, Tone::Five, "上字陽下字陰\n"),
        (Tone::Four, Tone::Three, Tone::Six, "上字陽下字陰\n"),
    ] {
        let result = get_faancit(
            &pronunciation("l", "ai", upper),
            &pronunciation("l", "ung", lower),
        );
        assert_eq!(result.pronunciation.tone(), expected);
        assert_eq!(result.diagnostics, diagnostic);
    }

    let adjusted = get_faancit(
        &pronunciation("l", "ai", Tone::Four),
        &pronunciation("f", "ung", Tone::Two),
    );
    assert_eq!(adjusted.pronunciation.tone(), Tone::Six);
    assert_eq!(
        adjusted.diagnostics,
        "上字陽下字陰\n上字陽聲而上字塞音或下字擦音\n"
    );
}

#[test]
fn exhaustive_rule_matrix_matches_reference_behavior() {
    for upper_initial in 0..INITIALS.len() as u8 {
        for upper_tone in 1..=6 {
            for lower_initial in 0..INITIALS.len() as u8 {
                for lower_tone in 1..=6 {
                    let upper_tone = Tone::try_from(upper_tone).unwrap();
                    let lower_tone = Tone::try_from(lower_tone).unwrap();
                    let upper = JyutpingChar::from_parts(upper_initial, 0, upper_tone).unwrap();
                    let lower = JyutpingChar::from_parts(lower_initial, 1, lower_tone).unwrap();
                    let (actual, actual_flags) = combine_packed(upper.packed(), lower.packed());

                    let mut expected_initial = upper.initial();
                    let mut expected_tone = lower_tone as u8;
                    let mut expected_flags = 0;
                    let upper_is_yin = upper_tone as u8 <= 3;
                    let lower_is_yin = lower_tone as u8 <= 3;
                    let upper_is_stop = ["b", "p", "d", "t", "g", "k", "gw", "kw", "z", "c"]
                        .contains(&upper.initial());

                    if !upper_is_yin && upper_is_stop {
                        expected_flags |= DIAG_UPPER_YANG_STOP;
                        if matches!(lower_tone, Tone::One | Tone::Four) {
                            expected_flags |= DIAG_ASPIRATED;
                            expected_initial = match upper.initial() {
                                "b" => "p",
                                "d" => "t",
                                "g" => "k",
                                "gw" => "kw",
                                "z" => "c",
                                value => value,
                            };
                        } else {
                            expected_flags |= DIAG_DEASPIRATED;
                            expected_initial = match upper.initial() {
                                "p" => "b",
                                "t" => "d",
                                "k" => "g",
                                "kw" => "gw",
                                "c" => "z",
                                value => value,
                            };
                        }
                    }
                    if upper.initial() == "f" {
                        expected_flags |= DIAG_LABIODENTAL;
                        expected_initial = "b";
                    }
                    if upper_is_yin && !lower_is_yin {
                        expected_flags |= DIAG_YIN_OVER_YANG;
                        expected_tone -= 3;
                    } else if !upper_is_yin && lower_is_yin {
                        expected_flags |= DIAG_YANG_OVER_YIN;
                        expected_tone += 3;
                    }
                    if !upper_is_yin
                        && (upper_is_stop || ["f", "s", "h"].contains(&lower.initial()))
                    {
                        expected_flags |= DIAG_YANG_ADJUSTMENT;
                        if expected_tone == 2 || expected_tone == 5 {
                            expected_tone += 1;
                        }
                    }

                    let actual = JyutpingChar::from_packed(actual.into()).unwrap();
                    assert_eq!(actual.initial(), expected_initial);
                    assert_eq!(actual.final_(), lower.final_());
                    assert_eq!(actual.tone() as u8, expected_tone);
                    assert_eq!(actual_flags, expected_flags);
                }
            }
        }
    }
}

#[test]
fn combines_packed_parts_without_allocation() {
    let upper = pronunciation("d", "ak", Tone::One);
    let lower = pronunciation("g", "ung", Tone::Four);
    let (packed, flags) = combine_raw(upper.packed().into(), lower.packed().into()).unwrap();
    let result = JyutpingChar::from_packed(packed.into()).unwrap();
    assert_eq!(result, pronunciation("d", "ung", Tone::One));
    assert_eq!(flags, DIAG_YIN_OVER_YANG);
}

#[test]
fn homophones_are_sorted_and_raw_iteration_resumes_after_previous() {
    let packed = pronunciation("d", "ung", Tone::One).packed();
    let expected: Vec<char> = homophones(packed).collect();
    assert!(expected.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(expected.contains(&'冬'));
    assert!(expected.contains(&'東'));

    let mut actual = Vec::new();
    let mut after = 0;
    while let Some(codepoint) = next_homophone(packed.into(), after) {
        actual.push(char::from_u32(codepoint).unwrap());
        after = codepoint;
    }
    assert_eq!(actual, expected);
    assert_eq!(next_homophone(packed.into(), u32::MAX), None);
}
