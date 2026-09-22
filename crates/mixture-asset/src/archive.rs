use crate::error::{AssetError, invalid, limit};
use std::ops::Range;

pub(crate) fn padded(size: usize) -> Result<usize, AssetError> {
    size.checked_add(511)
        .map(|n| n / 512 * 512)
        .ok_or_else(|| invalid("Entry size overflow."))
}
pub(crate) fn header(name: &str, size: usize) -> Result<[u8; 512], AssetError> {
    if name.len() > 99 || !name.is_ascii() || size as u64 > 0o77777777777 {
        return Err(invalid("Unrepresentable canonical header."));
    }
    let mut h = [0; 512];
    h[..name.len()].copy_from_slice(name.as_bytes());
    h[100..108].copy_from_slice(b"0000644\0");
    h[108..116].copy_from_slice(b"0000000\0");
    h[116..124].copy_from_slice(b"0000000\0");
    h[124..136].copy_from_slice(format!("{size:011o}\0").as_bytes());
    h[136..148].copy_from_slice(b"00000000000\0");
    h[148..156].fill(b' ');
    h[156] = b'0';
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    Ok(h)
}
pub(crate) fn entry(
    bytes: &[u8],
    cursor: &mut usize,
    name: &str,
    max: u64,
) -> Result<Range<usize>, AssetError> {
    let start = *cursor;
    let end = start
        .checked_add(512)
        .ok_or_else(|| invalid("Header offset overflow."))?;
    let raw = bytes
        .get(start..end)
        .ok_or_else(|| invalid("Truncated header.").evidence("offset", start as u64))?;
    let size_field = raw.get(124..135).ok_or_else(|| invalid("Missing size."))?;
    if !size_field.iter().all(|v| (b'0'..=b'7').contains(v)) {
        return Err(invalid("Noncanonical size."));
    }
    let size = size_field
        .iter()
        .try_fold(0u64, |n, b| {
            n.checked_mul(8)
                .and_then(|n| n.checked_add(u64::from(b - b'0')))
        })
        .ok_or_else(|| invalid("Size overflow."))?;
    let len = usize::try_from(size).map_err(|_| invalid("Host size overflow."))?;
    if raw != header(name, len)? {
        return Err(invalid("Noncanonical archive header.")
            .evidence("entry", name)
            .evidence("offset", start as u64));
    }
    limit("entryBytes", size, max)?;
    let data_end = end
        .checked_add(len)
        .ok_or_else(|| invalid("Entry overflow."))?;
    let next = end
        .checked_add(padded(len)?)
        .ok_or_else(|| invalid("Padding overflow."))?;
    let padding = bytes
        .get(data_end..next)
        .ok_or_else(|| invalid("Truncated entry."))?;
    if padding.iter().any(|b| *b != 0) {
        return Err(invalid("Nonzero entry padding."));
    }
    *cursor = next;
    Ok(end..data_end)
}
pub(crate) fn finish(bytes: &[u8], cursor: usize) -> Result<(), AssetError> {
    if bytes
        .get(cursor..)
        .is_none_or(|tail| tail.len() != 1024 || tail.iter().any(|b| *b != 0))
    {
        return Err(invalid(
            "Expected exactly two terminal zero blocks and EOF.",
        ));
    }
    Ok(())
}
pub(crate) fn append(out: &mut Vec<u8>, name: &str, data: &[u8]) -> Result<(), AssetError> {
    out.extend_from_slice(&header(name, data.len())?);
    out.extend_from_slice(data);
    let extra = padded(data.len())? - data.len();
    out.resize(out.len() + extra, 0);
    Ok(())
}
