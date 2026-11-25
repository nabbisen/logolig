//! The time of the ZIP's entries (RFC 024, job H3): the civil date from a day count, the clamping to
//! what the DOS field can hold, the even second, and the extended timestamp written beside it.
//!
//! The expected values come from outside this code: the Unix seconds of every date in the tables were
//! produced by Python's `calendar.timegm`, and a slow reference (counting years and months one at a
//! time) checks every day from 1970 to 2110.

use super::zip_time::{
    DOS_FIRST_UNIX, DOS_LAST_UNIX, civil_from_days, dos_time, extended_timestamp_payload, now_unix,
};
use super::*;

/// Seconds since the Unix epoch of a UTC date and time, by counting years and months one at a time
/// (a slow reference that shares no code with the function under test).
pub(super) fn reference_unix(y: u16, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> i64 {
    let mut days: i64 = 0;
    for year in 1970..y {
        days += if is_leap(year) { 366 } else { 365 };
    }
    for month in 1..mo {
        days += i64::from(days_in_month(y, month));
    }
    days += i64::from(d) - 1;
    days * 86_400 + i64::from(h) * 3600 + i64::from(mi) * 60 + i64::from(s)
}

pub(super) fn reference_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the Unix epoch")
        .as_secs() as i64
}

fn is_leap(y: u16) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn days_in_month(y: u16, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(y) => 29,
        _ => 28,
    }
}

/// (year, month, day, hour, minute, second).
type Fields = (u16, u8, u8, u8, u8, u8);

/// The fields of a DOS time.
fn fields(t: zip::DateTime) -> Fields {
    (
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
    )
}

/// Known instants (Unix seconds from `calendar.timegm`) and the DOS fields they give.
#[test]
fn known_instants_give_the_right_dos_fields() {
    let table: [(i64, Fields); 12] = [
        (315_532_800, (1980, 1, 1, 0, 0, 0)),
        (320_673_600, (1980, 2, 29, 12, 0, 0)),
        (951_782_400, (2000, 2, 29, 0, 0, 0)),
        (978_307_198, (2000, 12, 31, 23, 59, 58)),
        (978_307_200, (2001, 1, 1, 0, 0, 0)),
        (1_709_202_030, (2024, 2, 29, 10, 20, 30)),
        (1_735_689_598, (2024, 12, 31, 23, 59, 58)),
        // 2100 is not a leap year: the second after 02-28 23:59:58 is 03-01 00:00:00.
        (4_107_542_398, (2100, 2, 28, 23, 59, 58)),
        (4_107_542_400, (2100, 3, 1, 0, 0, 0)),
        (4_233_686_400, (2104, 2, 29, 0, 0, 0)),
        (1_795_680_668, (2026, 11, 26, 8, 11, 8)),
        (DOS_LAST_UNIX, (2107, 12, 31, 23, 59, 58)),
    ];
    for (unix, want) in table {
        assert_eq!(fields(dos_time(unix)), want, "unix {unix}");
    }
}

/// The DOS field stores seconds / 2: an odd second is rounded down, never up.
#[test]
fn an_odd_second_is_rounded_down_to_an_even_one() {
    // 2038-01-19 03:14:07 UTC, the last second of a signed 32-bit clock.
    assert_eq!(fields(dos_time(2_147_483_647)), (2038, 1, 19, 3, 14, 6));
    let minute = reference_unix(2024, 6, 1, 12, 30, 0);
    for second in 0..60u8 {
        let t = dos_time(minute + i64::from(second));
        assert_eq!(t.second(), second & !1, "second {second}");
        assert_eq!(t.minute(), 30);
    }
}

/// Outside 1980-01-01 .. 2107-12-31 23:59:58 the value is clamped to the ends: never a panic, never a
/// wrapped date.
#[test]
fn values_outside_the_range_clamp_to_its_ends() {
    let first = (1980, 1, 1, 0, 0, 0);
    let last = (2107, 12, 31, 23, 59, 58);
    for before in [0_i64, -1, 315_532_798, i64::MIN] {
        assert_eq!(fields(dos_time(before)), first, "unix {before}");
    }
    for beyond in [4_354_819_199, 4_354_819_200, 5_000_000_000, i64::MAX] {
        assert_eq!(fields(dos_time(beyond)), last, "unix {beyond}");
    }
    assert_eq!(DOS_FIRST_UNIX, reference_unix(1980, 1, 1, 0, 0, 0));
    assert_eq!(DOS_LAST_UNIX, reference_unix(2107, 12, 31, 23, 59, 58));
}

/// Every day from 1970-01-01 to 2110-12-31 against the slow reference: year, month and day.
#[test]
fn every_day_from_1970_to_2110_matches_a_slow_reference() {
    let (mut y, mut m, mut d) = (1970_i64, 1_u32, 1_u32);
    let mut days = 0_i64;
    while y <= 2110 {
        assert_eq!(civil_from_days(days), (y, m, d), "day {days}");
        days += 1;
        let year = u16::try_from(y).expect("a year");
        d += 1;
        if d > u32::from(days_in_month(year, u8::try_from(m).expect("a month"))) {
            d = 1;
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
    }
    assert!(days > 51_000, "{days} days were checked");
}

/// The clock being before the epoch cannot be set here; what the code does with it is the clamp above
/// (`now_unix` answers 0 for an error), and the reading itself never panics.
#[test]
fn reading_the_clock_gives_a_plausible_second_count() {
    let now = now_unix();
    assert!((now - reference_now()).abs() <= 2, "{now}");
    assert!(now > DOS_FIRST_UNIX, "the clock is after 1980");
}

/// The extended timestamp payload: flags 1 (modification time), then four little-endian bytes of UTC
/// seconds, held to what a signed 32-bit reader understands.
#[test]
fn the_extended_timestamp_payload_is_flags_and_little_endian_seconds() {
    assert_eq!(
        extended_timestamp_payload(0x5F9B_4C21),
        [1, 0x21, 0x4C, 0x9B, 0x5F]
    );
    assert_eq!(
        extended_timestamp_payload(1_795_680_668),
        [1, 156, 233, 7, 107]
    );
    assert_eq!(extended_timestamp_payload(-5), [1, 0, 0, 0, 0]);
    assert_eq!(
        extended_timestamp_payload(i64::MAX),
        [1, 0xFF, 0xFF, 0xFF, 0x7F]
    );
}

fn le16(b: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes([b[at], b[at + 1]]))
}

fn le32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn asset(name: &str, bytes: &[u8]) -> ResultAssetItem {
    ResultAssetItem {
        file_name: name.to_string(),
        relative_path: name.to_string(),
        bytes: bytes.to_vec(),
        kind: ResultAssetKind::Svg,
        dimensions: None,
        thumbnail: None,
    }
}

/// The `0x5455` field in an extra-field block: its payload, if present.
fn extended_timestamp_in(extra: &[u8]) -> Option<&[u8]> {
    let mut at = 0;
    while at + 4 <= extra.len() {
        let (id, len) = (le16(extra, at), le16(extra, at + 2));
        if id == 0x5455 {
            return Some(&extra[at + 4..at + 4 + len]);
        }
        at += 4 + len;
    }
    None
}

/// The written archive carries the field, tag `0x5455`, length 5, flags 1 and the UTC seconds, in the
/// **local** header and in the **central** directory of every entry; the two agree with the DOS time
/// to its resolution; and the rest of the archive is untouched (names, order, method, CRC, sizes).
#[test]
fn every_entry_carries_the_extended_timestamp_in_both_headers() {
    let items = vec![
        asset("favicon.svg", b"<svg/>"),
        asset("mono/favicon.ico", &[7u8; 300]),
    ];
    let before = reference_now();
    let bytes = build_zip_bytes(&items).expect("the zip builds");
    let after = reference_now();

    // Local headers: signature 0x04034b50, name length at 26, extra length at 28, name at 30.
    let mut at = 0;
    let mut local = Vec::new();
    for item in &items {
        assert_eq!(
            le32(&bytes, at),
            0x0403_4b50,
            "local header of {}",
            item.relative_path
        );
        let (name_len, extra_len) = (le16(&bytes, at + 26), le16(&bytes, at + 28));
        let name = &bytes[at + 30..at + 30 + name_len];
        assert_eq!(
            name,
            item.relative_path.as_bytes(),
            "entries keep their order and names"
        );
        let extra = &bytes[at + 30 + name_len..at + 30 + name_len + extra_len];
        local.push(
            extended_timestamp_in(extra)
                .expect("a local extended timestamp")
                .to_vec(),
        );
        let compressed = le32(&bytes, at + 18) as usize;
        at += 30 + name_len + extra_len + compressed;
    }
    // Central directory: signature 0x02014b50, name length at 28, extra length at 30, name at 46.
    let mut central = Vec::new();
    for item in &items {
        assert_eq!(
            le32(&bytes, at),
            0x0201_4b50,
            "central header of {}",
            item.relative_path
        );
        let (name_len, extra_len, comment_len) = (
            le16(&bytes, at + 28),
            le16(&bytes, at + 30),
            le16(&bytes, at + 32),
        );
        let extra = &bytes[at + 46 + name_len..at + 46 + name_len + extra_len];
        central.push(
            extended_timestamp_in(extra)
                .expect("a central extended timestamp")
                .to_vec(),
        );
        at += 46 + name_len + extra_len + comment_len;
    }
    for payload in local.iter().chain(central.iter()) {
        assert_eq!(payload.len(), 5, "length 5");
        assert_eq!(payload[0], 1, "flags: the modification time is present");
        let seconds = i64::from(le32(payload, 1));
        assert!(
            before <= seconds && seconds <= after,
            "UTC seconds {seconds} not in {before}..{after}"
        );
    }
    assert_eq!(local, central, "the local and central fields agree");

    // The archive still reads back as before (CRC and sizes checked by the reader).
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("readable");
    assert_eq!(archive.len(), 2);
    for item in &items {
        let mut entry = archive.by_name(&item.relative_path).expect("entry");
        assert_eq!(entry.compression(), zip::CompressionMethod::Deflated);
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).expect("CRC ok");
        assert_eq!(data, item.bytes);
    }
}

/// The application writes the ZIP on a blocking thread of a tokio runtime; the stamp works there.
#[test]
fn the_stamp_works_from_a_spawn_blocking_thread() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a runtime");
    let output = std::env::temp_dir().join(format!("logolig-zip-time-{}.zip", std::process::id()));
    let path = output.clone();
    runtime.block_on(async move {
        tokio::task::spawn_blocking(move || {
            write_zip_blocking(&path, &[asset("a.svg", b"<svg/>")])
        })
        .await
        .expect("the thread ran")
        .expect("the zip is written");
    });
    let file = std::fs::File::open(&output).expect("zip created");
    let mut archive = zip::ZipArchive::new(file).expect("zip readable");
    let year = archive
        .by_index(0)
        .expect("entry")
        .last_modified()
        .expect("time")
        .year();
    let _ = std::fs::remove_file(&output);
    assert!(year >= 2024, "stamped {year}, not the 1980 fallback");
}
