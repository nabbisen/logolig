//! The time stamped on the entries of the "Download all" ZIP (RFC 024, job H3).
//!
//! The application needs "now" as a date and time and has no time package for it: `std` gives
//! seconds since the Unix epoch (UTC), and the civil date follows from the day count by a small
//! function of ours (the days-from-civil algorithm of Howard Hinnant, run backwards). No
//! `unsafe`, no dependency, no zone database: **UTC only**.
//!
//! A ZIP entry carries its time in an MS-DOS field that has **no time zone** and a resolution of
//! two seconds, and covers 1980-01-01 00:00:00 to 2107-12-31 23:59:58. A value outside that range is
//! clamped to its ends; an odd second is rounded down. Extractors that read only that field show
//! the UTC time as if it were local; those that understand the extended timestamp (`0x5455`,
//! Unix seconds, UTC) show the right instant.

/// 1980-01-01 00:00:00 UTC, the first instant a ZIP's DOS time can hold.
pub(super) const DOS_FIRST_UNIX: i64 = 315_532_800;
/// 2107-12-31 23:59:58 UTC, the last one.
pub(super) const DOS_LAST_UNIX: i64 = 4_354_819_198;
/// The header id of the extended timestamp extra field ("UT").
pub(super) const EXTENDED_TIMESTAMP_ID: u16 = 0x5455;

/// Seconds since 1970-01-01 00:00:00 UTC by the system clock. A clock set before the epoch gives 0
/// (which then clamps to the ZIP's first instant): never a panic.
pub(super) fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// The civil date (year, month 1-12, day 1-31) of the day `days` days after 1970-01-01, in the
/// proleptic Gregorian calendar. Valid for every `i64` day count that does not overflow the
/// arithmetic (the caller clamps to the ZIP's range first).
pub(super) fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Shift the epoch to 0000-03-01, so that the leap day is the last day of the "year".
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097); // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365; // [0, 399]
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]
    let month_index = (5 * day_of_year + 2) / 153; // 0 = March, ..., 11 = February
    let day = day_of_year - (153 * month_index + 2) / 5 + 1; // [1, 31]
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    }; // [1, 12]
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

/// The ZIP's DOS time for `unix` seconds: clamped to 1980-01-01 .. 2107-12-31, the second rounded
/// down to an even one (the field stores seconds / 2).
pub(super) fn dos_time(unix: i64) -> zip::DateTime {
    let unix = unix.clamp(DOS_FIRST_UNIX, DOS_LAST_UNIX);
    let (year, month, day) = civil_from_days(unix.div_euclid(86_400));
    let seconds = unix.rem_euclid(86_400);
    let field = |v: i64| u8::try_from(v).unwrap_or(0);
    u16::try_from(year)
        .ok()
        .and_then(|year| {
            zip::DateTime::from_date_and_time(
                year,
                field(i64::from(month)),
                field(i64::from(day)),
                field(seconds / 3_600),
                field(seconds % 3_600 / 60),
                field(seconds % 60),
            )
            .ok()
        })
        .unwrap_or_default()
}

/// The payload of the extended timestamp field for `unix` seconds: flags `1` (a modification time
/// follows) and the time as a little-endian 32-bit count of seconds since the epoch. The count is
/// held to what a signed 32-bit field means to every reader (0 to 2038-01-19 03:14:07).
pub(super) fn extended_timestamp_payload(unix: i64) -> [u8; 5] {
    let seconds = u32::try_from(unix.clamp(0, i64::from(i32::MAX))).unwrap_or(0);
    let mut payload = [1, 0, 0, 0, 0];
    payload[1..].copy_from_slice(&seconds.to_le_bytes());
    payload
}
