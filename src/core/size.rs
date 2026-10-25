//! Declared sizes, in bytes.
//!
//! **Only the declared side is ever parsed.** Every backend that reports a current size is asked
//! for raw bytes — `zfs list -p`, `lvs --units b --nosuffix`, `btrfs qgroup show --raw` — so the
//! comparison is `u64` against `u64` and no tool's display formatting is in the loop. That is the
//! whole design: `btrfs qgroup show` prints `10.00GiB`, `lvs` prints `10.00g`, and `zfs list`
//! prints `10G` for the same number, and a comparator that had to understand all three would
//! report a change on every sync for ever the first time one of them rounded.

/// A declared size (`10G`, `100M`, `1.5GiB`, `10737418240`) in bytes, or `None` if it is not one.
///
/// Units are powers of 1024, because that is what all three tools mean by `G`: `lvcreate -L 10G`,
/// `zfs set quota=10G` and `btrfs qgroup limit 10G` each reserve 10 GiB. Reading `G` as a
/// thousand million here would make every declared size disagree with the volume it created.
pub fn parse_size(declared: &str) -> Option<u64> {
    let s = declared.trim();
    if s.is_empty() {
        return None;
    }
    let digits_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    let (number, suffix) = s.split_at(digits_end);
    let value: f64 = number.parse().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let suffix = suffix.trim();
    // `10G`, `10GB` and `10GiB` are one size written three ways. The letter is the magnitude and
    // the rest is punctuation — LVM accepts a bare `G`, ZFS accepts `GB`, and `btrfs qgroup show`
    // prints `GiB`.
    let letter = suffix.chars().next().map(|c| c.to_ascii_uppercase());
    let rest = suffix.get(letter.map_or(0, |_| 1)..).unwrap_or("");
    if !matches!(rest.to_ascii_uppercase().as_str(), "" | "B" | "IB") {
        return None;
    }
    let scale: u64 = match letter {
        None => 1,
        Some('B') if rest.is_empty() => 1,
        Some('K') => 1 << 10,
        Some('M') => 1 << 20,
        Some('G') => 1 << 30,
        Some('T') => 1 << 40,
        Some('P') => 1 << 50,
        Some('E') => 1 << 60,
        _ => return None,
    };
    let bytes = value * scale as f64;
    if bytes > u64::MAX as f64 {
        return None;
    }
    Some(bytes as u64)
}

/// Bytes as the shortest exact-enough form a person reads, for an error that names two sizes.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [(u64, &str); 5] = [
        (1 << 50, "P"),
        (1 << 40, "T"),
        (1 << 30, "G"),
        (1 << 20, "M"),
        (1 << 10, "K"),
    ];
    for (scale, unit) in UNITS {
        if bytes >= scale {
            let whole = bytes as f64 / scale as f64;
            return if (whole - whole.round()).abs() < f64::EPSILON {
                format!("{}{}", whole.round() as u64, unit)
            } else {
                format!("{:.2}{}", whole, unit)
            };
        }
    }
    format!("{}B", bytes)
}

/// Whether a declared size and a reported byte count describe the same volume.
///
/// A declaration that cannot be parsed is **not** drift. The alternative is a sync that reports a
/// change it can never carry out, every run, for a line the tool will refuse anyway — and the
/// refusal is the backend's to make, where it can name the value.
pub fn same_size(declared: &str, actual_bytes: u64) -> bool {
    parse_size(declared).is_none_or(|want| want == actual_bytes)
}
