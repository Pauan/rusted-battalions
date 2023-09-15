#[cfg(feature = "unicode")]
mod unicode {
    pub(crate) fn char_offset(c: char, width: u32) -> f32 {
        fn is_centered(c: char) -> bool {
            // Data is obtained from the Unifont font/plane00/plane00-combining.txt file
            match c {
                '\u{0488}'..='\u{0489}' |
                '\u{0900}'..='\u{0903}' |
                '\u{093A}'..='\u{093C}' |
                '\u{093E}'..='\u{094F}' |
                '\u{0951}'..='\u{0957}' |
                '\u{0962}'..='\u{0963}' |
                '\u{0981}'..='\u{0983}' |
                '\u{09BC}' |
                '\u{09BE}'..='\u{09C4}' |
                '\u{09C7}'..='\u{09C8}' |
                '\u{09CB}'..='\u{09CD}' |
                '\u{09D7}' |
                '\u{09E2}'..='\u{09E3}' |
                '\u{09FE}' |
                '\u{0A01}'..='\u{0A03}' |
                '\u{0A3C}' |
                '\u{0A3E}'..='\u{0A42}' |
                '\u{0A47}'..='\u{0A48}' |
                '\u{0A4B}'..='\u{0A4D}' |
                '\u{0A51}' |
                '\u{0A70}'..='\u{0A71}' |
                '\u{0A75}' |
                '\u{0A81}'..='\u{0A83}' |
                '\u{0ABC}' |
                '\u{0ABE}'..='\u{0AC5}' |
                '\u{0AC7}'..='\u{0AC9}' |
                '\u{0ACB}'..='\u{0ACD}' |
                '\u{0AE2}'..='\u{0AE3}' |
                '\u{0AFA}'..='\u{0AFF}' |
                '\u{0B01}'..='\u{0B03}' |
                '\u{0B3C}' |
                '\u{0B3E}'..='\u{0B44}' |
                '\u{0B47}'..='\u{0B48}' |
                '\u{0B4B}'..='\u{0B4D}' |
                '\u{0B55}'..='\u{0B57}' |
                '\u{0B62}'..='\u{0B63}' |
                '\u{0B82}' |
                '\u{0BBE}'..='\u{0BC2}' |
                '\u{0BC6}'..='\u{0BC8}' |
                '\u{0BCA}'..='\u{0BCD}' |
                '\u{0BD7}' |
                '\u{0C00}'..='\u{0C04}' |
                '\u{0C3C}' |
                '\u{0C3E}'..='\u{0C44}' |
                '\u{0C46}'..='\u{0C48}' |
                '\u{0C4A}'..='\u{0C4D}' |
                '\u{0C55}'..='\u{0C56}' |
                '\u{0C62}'..='\u{0C63}' |
                '\u{0C81}'..='\u{0C83}' |
                '\u{0CBC}' |
                '\u{0CBE}'..='\u{0CC4}' |
                '\u{0CC6}'..='\u{0CC8}' |
                '\u{0CCA}'..='\u{0CCD}' |
                '\u{0CD5}'..='\u{0CD6}' |
                '\u{0CE2}'..='\u{0CE3}' |
                '\u{0D00}'..='\u{0D03}' |
                '\u{0D3B}'..='\u{0D3C}' |
                '\u{0D3E}'..='\u{0D44}' |
                '\u{0D46}'..='\u{0D48}' |
                '\u{0D4A}'..='\u{0D4D}' |
                '\u{0D57}' |
                '\u{0D62}'..='\u{0D63}' |
                '\u{0D81}'..='\u{0D83}' |
                '\u{0DCA}' |
                '\u{0DCF}'..='\u{0DD4}' |
                '\u{0DD6}' |
                '\u{0DD8}'..='\u{0DDF}' |
                '\u{0DF2}'..='\u{0DF3}' |
                '\u{0F18}'..='\u{0F19}' |
                '\u{0F35}' |
                '\u{0F37}' |
                '\u{0F39}' |
                '\u{0F3E}'..='\u{0F3F}' |
                '\u{0F71}'..='\u{0F84}' |
                '\u{0F86}'..='\u{0F87}' |
                '\u{0F8D}'..='\u{0F97}' |
                '\u{0F99}'..='\u{0FBC}' |
                '\u{0FC6}' |
                '\u{102B}'..='\u{103E}' |
                '\u{1056}'..='\u{1059}' |
                '\u{105E}'..='\u{1060}' |
                '\u{1062}'..='\u{1064}' |
                '\u{1067}'..='\u{106D}' |
                '\u{1071}'..='\u{1074}' |
                '\u{1082}'..='\u{1083}' |
                '\u{1085}'..='\u{108D}' |
                '\u{108F}' |
                '\u{109A}'..='\u{109D}' |
                '\u{135D}'..='\u{135F}' |
                '\u{1712}'..='\u{1715}' |
                '\u{1732}'..='\u{1734}' |
                '\u{1752}'..='\u{1753}' |
                '\u{1772}'..='\u{1773}' |
                '\u{17B6}'..='\u{17BA}' |
                '\u{17BC}'..='\u{17C0}' |
                '\u{17C4}'..='\u{17CA}' |
                '\u{17CC}'..='\u{17D3}' |
                '\u{17DD}' |
                '\u{18A9}' |
                '\u{1920}'..='\u{192B}' |
                '\u{1930}'..='\u{193B}' |
                '\u{1A17}'..='\u{1A1B}' |
                '\u{1A55}'..='\u{1A5E}' |
                '\u{1A60}'..='\u{1A7C}' |
                '\u{1A7F}' |
                '\u{1B00}'..='\u{1B04}' |
                '\u{1B34}'..='\u{1B44}' |
                '\u{1B6B}'..='\u{1B73}' |
                '\u{1B80}'..='\u{1B82}' |
                '\u{1BA1}'..='\u{1BAD}' |
                '\u{1BE6}'..='\u{1BF3}' |
                '\u{1C24}'..='\u{1C37}' |
                '\u{1CD0}'..='\u{1CD2}' |
                '\u{1CD4}'..='\u{1CE8}' |
                '\u{1CED}' |
                '\u{1CF4}' |
                '\u{1CF7}'..='\u{1CF9}' |
                '\u{20DD}'..='\u{20E0}' |
                '\u{20E2}'..='\u{20E4}' |
                '\u{20E7}' |
                '\u{20EA}' |
                '\u{302A}'..='\u{302F}' |
                '\u{3099}'..='\u{309A}' |
                '\u{A670}' |
                '\u{A672}' |
                '\u{A802}' |
                '\u{A806}' |
                '\u{A80B}' |
                '\u{A823}'..='\u{A827}' |
                '\u{A82C}' |
                '\u{A880}'..='\u{A881}' |
                '\u{A8B4}'..='\u{A8C5}' |
                '\u{A8E0}'..='\u{A8F1}' |
                '\u{A8FF}' |
                '\u{A926}'..='\u{A92D}' |
                '\u{A947}'..='\u{A953}' |
                '\u{A980}'..='\u{A983}' |
                '\u{A9B3}'..='\u{A9C0}' |
                '\u{AA29}'..='\u{AA36}' |
                '\u{AA43}' |
                '\u{AA4C}'..='\u{AA4D}' |
                '\u{AA7B}'..='\u{AA7D}' |
                '\u{AAB0}' |
                '\u{AAB2}'..='\u{AAB4}' |
                '\u{AAB7}'..='\u{AAB8}' |
                '\u{AABE}'..='\u{AABF}' |
                '\u{AAC1}' |
                '\u{AAEB}'..='\u{AAEF}' |
                '\u{AAF5}'..='\u{AAF6}' |
                '\u{ABE3}'..='\u{ABEA}' |
                '\u{ABEC}' => true,
                _ => false,
            }
        }

        if is_centered(c) {
            if width == 2 {
                0.0

            } else {
                -0.5
            }

        } else if width == 2 {
            0.5

        } else {
            0.0
        }
    }

    #[inline]
    pub(crate) fn char_width(c: char) -> u32 {
        use unicode_width::UnicodeWidthChar;

        c.width().unwrap_or(2) as u32
    }

    #[inline]
    pub(crate) fn graphemes(s: &str) -> impl Iterator<Item = &str> {
        use unicode_segmentation::UnicodeSegmentation;

        s.graphemes(true)
    }
}


#[cfg(not(feature = "unicode"))]
mod ascii {
    #[inline]
    pub(crate) fn char_offset(_c: char, _width: u32) -> f32 {
        0.0
    }

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

    #[inline]
    pub(crate) fn graphemes(s: &str) -> impl Iterator<Item = &str> {
        s.split_inclusive(|_| true)
    }
}


#[cfg(feature = "unicode")]
pub(crate) use unicode::*;

#[cfg(not(feature = "unicode"))]
pub(crate) use ascii::*;
