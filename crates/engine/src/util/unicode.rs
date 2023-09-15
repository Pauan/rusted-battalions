#[cfg(feature = "unicode")]
#[inline]
pub(crate) fn char_width(c: char) -> u32 {
    use unicode_width::UnicodeWidthChar;

    c.width().unwrap_or(2) as u32
}

#[cfg(feature = "unicode")]
#[inline]
pub(crate) fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    use unicode_segmentation::UnicodeSegmentation;

    s.graphemes(true)
}


#[cfg(not(feature = "unicode"))]
#[inline]
pub(crate) fn char_width(c: char) -> u32 {
    // Simple version of unicode-width which only supports ASCII
    match c as u32 {
        _c @ 0 => 2,
        cu if cu < 0x20 => 2,
        cu if cu < 0x7F => 1,
        cu if cu < 0xA0 => 2,
        _ => 2,
    }
}

#[cfg(not(feature = "unicode"))]
#[inline]
pub(crate) fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    s.split_inclusive(|_| true)
}
