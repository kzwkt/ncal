//! Festivals and public holidays, loaded from an embedded dataset.
//!
//! Most Nepali festivals are tithi-based (lunar), so they cannot be derived from
//! `BS_REFERENCE` the way month lengths can. Three entry kinds cover the ground:
//!
//! * `fixed`    - same BS month/day every year (Nepali New Year, Constitution Day)
//! * `ad_fixed` - same Gregorian month/day every year (Labour Day, Christmas)
//! * `lunar`    - pinned to one specific BS year, hand-entered per year
//!
//! The dataset is embedded with `include_str!`, so there is no runtime file
//! dependency and no network access. It is parsed once, lazily.

use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::BsDate;

const FESTIVAL_DATA: &str = include_str!("data/festivals.json");

/// A single festival or public holiday occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Festival {
    /// Name in English / romanized Nepali.
    pub en: String,
    /// Name in Devanagari.
    pub np: String,
    /// Whether this is a gazetted public holiday, as opposed to an observance.
    #[serde(default)]
    pub holiday: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FixedEntry {
    month: u8,
    day: u8,
    #[serde(flatten)]
    festival: Festival,
}

#[derive(Debug, Clone, Deserialize)]
struct AdFixedEntry {
    ad_month: u32,
    ad_day: u32,
    #[serde(flatten)]
    festival: Festival,
}

#[derive(Debug, Clone, Deserialize)]
struct LunarEntry {
    year: u16,
    month: u8,
    day: u8,
    #[serde(flatten)]
    festival: Festival,
}

#[derive(Debug, Clone, Deserialize)]
struct FestivalData {
    /// Inclusive `[first, last]` BS years for which lunar entries were entered.
    /// Callers surface this so a user paging past it knows the grid is thinner
    /// there rather than assuming the year genuinely has no festivals.
    #[serde(default)]
    lunar_years_covered: Vec<u16>,
    #[serde(default)]
    fixed: Vec<FixedEntry>,
    #[serde(default)]
    ad_fixed: Vec<AdFixedEntry>,
    #[serde(default)]
    lunar: Vec<LunarEntry>,
}

fn data() -> &'static FestivalData {
    static DATA: OnceLock<FestivalData> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(FESTIVAL_DATA).expect("embedded festivals.json must be valid")
    })
}

/// Inclusive BS year range for which lunar (tithi-based) festivals are present,
/// as `(first, last)`. `None` when the dataset carries no lunar entries at all.
pub fn lunar_years_covered() -> Option<(u16, u16)> {
    let years = &data().lunar_years_covered;
    match years.len() {
        0 => None,
        1 => Some((years[0], years[0])),
        _ => Some((years[0], years[1])),
    }
}

/// Every festival falling on one day. `ad` is the Gregorian date the BS date maps
/// to, needed for the AD-fixed holidays.
///
/// A BS year with no hand-entered lunar data is not an error: the fixed-date
/// entries still come back.
pub fn festivals_on(bs: BsDate, ad: NaiveDate) -> Vec<Festival> {
    let data = data();
    let mut found = Vec::new();

    for entry in &data.fixed {
        if entry.month == bs.month && entry.day == bs.day {
            found.push(entry.festival.clone());
        }
    }

    for entry in &data.ad_fixed {
        if entry.ad_month == ad.month() && entry.ad_day == ad.day() {
            found.push(entry.festival.clone());
        }
    }

    for entry in &data.lunar {
        if entry.year == bs.year && entry.month == bs.month && entry.day == bs.day {
            found.push(entry.festival.clone());
        }
    }

    found
}

/// Every festival in one BS month, as `(bs_day, festival)` sorted by day.
///
/// AD-fixed holidays are resolved by walking the month, so a Gregorian date that
/// lands mid-month is picked up correctly.
pub fn festivals_in_month(year: u16, month: u8) -> Vec<(u8, Festival)> {
    let Ok(month_len) = crate::bs_month_len(year, month) else {
        return Vec::new();
    };

    let mut found = Vec::new();
    for day in 1..=month_len {
        let bs = BsDate { year, month, day };
        let Ok(ad) = crate::bs_to_ad(bs) else {
            continue;
        };
        for festival in festivals_on(bs, ad) {
            found.push((day, festival));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bs_to_ad;

    #[test]
    fn embedded_dataset_parses() {
        // `data()` panics on malformed JSON; touching it is the test.
        assert!(!data().fixed.is_empty());
    }

    #[test]
    fn new_year_is_found_in_every_year() {
        for year in [2080u16, 2083, 2090] {
            let bs = BsDate {
                year,
                month: 1,
                day: 1,
            };
            let ad = bs_to_ad(bs).unwrap();
            let names: Vec<_> = festivals_on(bs, ad).into_iter().map(|f| f.en).collect();
            assert!(
                names.iter().any(|n| n.contains("New Year")),
                "BS {year}-01-01 should carry Nepali New Year, got {names:?}"
            );
        }
    }

    #[test]
    fn a_year_without_lunar_data_still_returns_fixed_festivals() {
        // BS 2005 predates any hand-entered lunar data; it must not error out.
        let bs = BsDate {
            year: 2005,
            month: 1,
            day: 1,
        };
        let ad = bs_to_ad(bs).unwrap();
        assert!(!festivals_on(bs, ad).is_empty());
    }

    #[test]
    fn ordinary_days_have_no_festivals() {
        let bs = BsDate {
            year: 2083,
            month: 1,
            day: 4,
        };
        let ad = bs_to_ad(bs).unwrap();
        assert!(festivals_on(bs, ad).is_empty());
    }

    #[test]
    fn month_lookup_is_sorted_and_bounded() {
        let found = festivals_in_month(2083, 1);
        assert!(found.iter().any(|(day, _)| *day == 1));
        assert!(found.windows(2).all(|w| w[0].0 <= w[1].0));
        let month_len = crate::bs_month_len(2083, 1).unwrap();
        assert!(found.iter().all(|(day, _)| *day >= 1 && *day <= month_len));
    }

    #[test]
    fn out_of_range_month_is_empty_not_a_panic() {
        assert!(festivals_in_month(2083, 13).is_empty());
        assert!(festivals_in_month(9999, 1).is_empty());
    }
}
