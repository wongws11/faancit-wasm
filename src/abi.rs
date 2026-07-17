use crate::jyutping;

/// Returned for an absent lookup, invalid packed value, or exhausted iteration.
pub const ABSENT: u32 = u32::MAX;

// `faancit_combine` places these flags at bits 16..22. Browser code must check
// them in this order to reproduce the native diagnostic log exactly.
/// `上字陽聲塞音\n`
pub const DIAG_UPPER_YANG_STOP: u32 = 1 << 16;
/// `下字平聲則聲母送氣\n`
pub const DIAG_ASPIRATED: u32 = 1 << 17;
/// `下字仄聲則聲母不送氣\n`
pub const DIAG_DEASPIRATED: u32 = 1 << 18;
/// `古無輕唇音\n`
pub const DIAG_LABIODENTAL: u32 = 1 << 19;
/// `上字陰下字陽\n`
pub const DIAG_YIN_OVER_YANG: u32 = 1 << 20;
/// `上字陽下字陰\n`
pub const DIAG_YANG_OVER_YIN: u32 = 1 << 21;
/// `上字陽聲而上字塞音或下字擦音\n`
pub const DIAG_YANG_ADJUSTMENT: u32 = 1 << 22;

#[unsafe(export_name = "l")]
pub extern "C" fn faancit_lookup(codepoint: u32) -> u32 {
    jyutping::lookup_codepoint(codepoint)
        .map(|value| u32::from(value.packed()))
        .unwrap_or(ABSENT)
}

/// Combines two validated packed pronunciations.
///
/// The result pronunciation occupies bits 0..15 and diagnostic flags occupy
/// bits 16..22. Invalid inputs return [`ABSENT`].
#[unsafe(export_name = "c")]
pub extern "C" fn faancit_combine(upper: u32, lower: u32) -> u32 {
    jyutping::combine_raw(upper, lower)
        .map(|(packed, diagnostics)| u32::from(packed) | (u32::from(diagnostics) << 16))
        .unwrap_or(ABSENT)
}

/// Returns the next sorted homophone after `after_codepoint`.
///
/// Start iteration with zero, then pass each returned codepoint back as
/// `after_codepoint` until [`ABSENT`] is returned.
#[unsafe(export_name = "h")]
pub extern "C" fn faancit_next_homophone(packed: u32, after_codepoint: u32) -> u32 {
    jyutping::next_homophone(packed, after_codepoint).unwrap_or(ABSENT)
}
