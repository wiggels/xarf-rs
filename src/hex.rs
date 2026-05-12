//! Lower-case hex encoder for digest output.
//!
//! Written byte-at-a-time into a pre-sized `String` via a 16-entry lookup
//! table — no per-byte heap allocations.

const TABLE: &[u8; 16] = b"0123456789abcdef";

/// Encode `bytes` as lower-case hex. Result length is `bytes.len() * 2`.
pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(TABLE[(b >> 4) as usize]);
        out.push(TABLE[(b & 0x0f) as usize]);
    }
    // SAFETY: TABLE contains only ASCII digits 0-9a-f, so every pushed byte is
    // valid UTF-8.
    unsafe { String::from_utf8_unchecked(out) }
}
