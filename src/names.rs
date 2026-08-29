//! Month, weekday and numeral names for the Bikram Sambat calendar.
//!
//! Everything here is pure lookup: no dates, no conversion. The Latin names are
//! the ones `ncal` has always printed; the Devanagari names and numerals are what
//! a Nepali reader actually expects to see.

/// Romanized BS month name. `month` is 1-12; anything else yields `""`.
pub fn bs_month_name(month: u8) -> &'static str {
    match month {
        1 => "Baisakh",
        2 => "Jestha",
        3 => "Asar",
        4 => "Shrawan",
        5 => "Bhadra",
        6 => "Ashwin",
        7 => "Kartik",
        8 => "Mangsir",
        9 => "Poush",
        10 => "Magh",
        11 => "Falgun",
        12 => "Chaitra",
        _ => "",
    }
}

/// BS month name in Devanagari. `month` is 1-12; anything else yields `""`.
pub fn bs_month_name_np(month: u8) -> &'static str {
    match month {
        1 => "बैशाख",
        2 => "जेठ",
        3 => "असार",
        4 => "साउन",
        5 => "भदौ",
        6 => "असोज",
        7 => "कात्तिक",
        8 => "मंसिर",
        9 => "पुष",
        10 => "माघ",
        11 => "फागुन",
        12 => "चैत",
        _ => "",
    }
}

/// Weekday name, `weekday` counted from Sunday = 0 to match
/// `chrono::Weekday::num_days_from_sunday` and the `Su Mo Tu ...` header.
pub fn weekday_name(weekday: u8) -> &'static str {
    match weekday {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "",
    }
}

/// Weekday name in Devanagari, Sunday = 0.
pub fn weekday_name_np(weekday: u8) -> &'static str {
    match weekday {
        0 => "आइतबार",
        1 => "सोमबार",
        2 => "मंगलबार",
        3 => "बुधबार",
        4 => "बिहीबार",
        5 => "शुक्रबार",
        6 => "शनिबार",
        _ => "",
    }
}

/// Short Devanagari weekday label, for a calendar's column header. Sunday = 0.
pub fn weekday_short_np(weekday: u8) -> &'static str {
    match weekday {
        0 => "आइत",
        1 => "सोम",
        2 => "मंगल",
        3 => "बुध",
        4 => "बिहि",
        5 => "शुक्र",
        6 => "शनि",
        _ => "",
    }
}

/// Two-letter Latin weekday label, matching the `Su Mo Tu We Th Fr Sa` header.
pub fn weekday_short(weekday: u8) -> &'static str {
    match weekday {
        0 => "Su",
        1 => "Mo",
        2 => "Tu",
        3 => "We",
        4 => "Th",
        5 => "Fr",
        6 => "Sa",
        _ => "",
    }
}

const DEVANAGARI_DIGITS: [char; 10] = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];

/// Render a number with Devanagari digits: `13` -> `"१३"`.
pub fn to_devanagari(n: u32) -> String {
    if n == 0 {
        return DEVANAGARI_DIGITS[0].to_string();
    }

    let mut digits = Vec::new();
    let mut rest = n;
    while rest > 0 {
        digits.push(DEVANAGARI_DIGITS[(rest % 10) as usize]);
        rest /= 10;
    }
    digits.iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_names_cover_the_whole_year() {
        for month in 1..=12u8 {
            assert!(!bs_month_name(month).is_empty());
            assert!(!bs_month_name_np(month).is_empty());
        }
        assert_eq!(bs_month_name(0), "");
        assert_eq!(bs_month_name_np(13), "");
    }

    #[test]
    fn weekday_names_cover_the_whole_week() {
        for weekday in 0..=6u8 {
            assert!(!weekday_name(weekday).is_empty());
            assert!(!weekday_name_np(weekday).is_empty());
            assert!(!weekday_short_np(weekday).is_empty());
            assert_eq!(weekday_short(weekday).chars().count(), 2);
        }
        assert_eq!(weekday_name(7), "");
    }

    #[test]
    fn devanagari_digits_render() {
        assert_eq!(to_devanagari(0), "०");
        assert_eq!(to_devanagari(7), "७");
        assert_eq!(to_devanagari(13), "१३");
        assert_eq!(to_devanagari(2083), "२०८३");
    }

    #[test]
    fn devanagari_is_one_char_per_digit() {
        // Guards the `center_text` alignment assumption: these are single
        // scalar values, so `.chars().count()` is the right width measure.
        assert_eq!(to_devanagari(2083).chars().count(), 4);
        assert_eq!(to_devanagari(31).chars().count(), 2);
    }
}
