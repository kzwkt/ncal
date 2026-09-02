pub mod app;
pub mod cli;
pub mod config;
pub mod festivals;
pub mod grid;
pub mod json;
pub mod names;
pub mod panchanga;

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Used for offsetting the BS date using the AD date.
//
// The first 12 elements are the length of each BS month; the 13th is the year
// total. There is no formula for these: a BS month runs from one solar sankranti
// to the next, and the lengths are fixed year by year by Nepal's Calendar
// Determination Committee. They have to come from a published table.
//
// Source: cross-checked against two independent published tables that agree for
// every year here except BS 2089 —
//   khumnath/nepdate            src/bsmonthdata.cpp   BS 2000-2089
//   remotemerge/nepali-date-converter  html/src/years.ts  BS 1975-2099
// BS 2089 is taken from the latter, which is also the only source covering
// 2090-2099. Rows are never extrapolated; when a year is not in a source, the
// table stops rather than guessing.
#[allow(unused)]
pub const BS_REFERENCE: [[u16; 13]; 100] = [
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2000
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2001
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2002
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2003
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2004
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2005
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2006
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2007
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365], // BS 2008
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2009
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2010
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2011
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2012
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2013
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2014
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2015
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2016
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2017
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2018
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2019
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2020
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2021
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365], // BS 2022
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2023
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2024
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2025
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2026
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2027
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2028
    [31, 31, 32, 31, 32, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2029
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2030
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2031
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2032
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2033
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2034
    [30, 32, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365], // BS 2035
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2036
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2037
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2038
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2039
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2040
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2041
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2042
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2043
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2044
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2045
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2046
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2047
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2048
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365], // BS 2049
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2050
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2051
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2052
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365], // BS 2053
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2054
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2055
    [31, 31, 32, 31, 32, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2056
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2057
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2058
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2059
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2060
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2061
    [30, 32, 31, 32, 31, 31, 29, 30, 29, 30, 29, 31, 365], // BS 2062
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2063
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2064
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2065
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365], // BS 2066
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2067
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2068
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2069
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2070
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2071
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2072
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2073
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2074
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2075
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365], // BS 2076
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2077
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2078
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2079
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365], // BS 2080
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366], // BS 2081
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2082
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2083
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2084
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2085
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2086
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2087
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2088
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365], // BS 2089
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2090
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2091
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2092
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365], // BS 2093
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2094
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2095
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366], // BS 2096
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365], // BS 2097
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365], // BS 2098
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365], // BS 2099
];

pub const BS_START_YEAR: u16 = 2000;
pub const BS_END_YEAR: u16 = BS_START_YEAR + BS_REFERENCE.len() as u16 - 1;
const BS_ANCHOR_AD_YEAR: i32 = 1943;
const BS_ANCHOR_AD_MONTH: u32 = 4;
const BS_ANCHOR_AD_DAY: u32 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Error)]
// uwu<3
pub enum NcalError {
    #[error("Invalid argument; expected 0-3 arguments, got {arg}")]
    InvalidArgumentLength { arg: String },

    #[error("Missing year")]
    MissingYear,

    #[error("Invalid year; choose between {start} and {end}, got {got}")]
    InvalidYearRange { start: u16, end: u16, got: u32 },

    #[error("Invalid month; choose between 1-12")]
    InvalidMonth,

    #[error("Invalid day")]
    InvalidDay,

    #[error("Date is outside supported AD conversion range")]
    UnsupportedAdDate,

    #[error("Invalid date {got:?}; expected YYYY-MM-DD")]
    InvalidDateFormat { got: String },
}

fn ad_anchor_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(BS_ANCHOR_AD_YEAR, BS_ANCHOR_AD_MONTH, BS_ANCHOR_AD_DAY)
        .expect("anchor date must be valid")
}

pub fn bs_month_len(year: u16, month: u8) -> Result<u8, NcalError> {
    if !(BS_START_YEAR..=BS_END_YEAR).contains(&year) {
        return Err(NcalError::InvalidYearRange {
            start: BS_START_YEAR,
            end: BS_END_YEAR,
            got: year as u32,
        });
    }

    if !(1..=12).contains(&month) {
        return Err(NcalError::InvalidMonth);
    }

    let year_idx = (year - BS_START_YEAR) as usize;
    Ok(BS_REFERENCE[year_idx][(month - 1) as usize] as u8)
}

pub fn validate_bs_date(bs_date: BsDate) -> Result<(), NcalError> {
    let month_len = bs_month_len(bs_date.year, bs_date.month)?;
    if bs_date.day == 0 || bs_date.day > month_len {
        return Err(NcalError::InvalidDay);
    }
    Ok(())
}

pub fn bs_to_ad(bs_date: BsDate) -> Result<NaiveDate, NcalError> {
    validate_bs_date(bs_date)?;

    let year_idx = (bs_date.year - BS_START_YEAR) as usize;
    let offset_year_days: i64 = BS_REFERENCE
        .iter()
        .take(year_idx)
        .map(|row| row[12] as i64)
        .sum();

    let offset_month_days: i64 = BS_REFERENCE[year_idx]
        .iter()
        .take((bs_date.month - 1) as usize)
        .map(|days| *days as i64)
        .sum();

    let total_offset = offset_year_days + offset_month_days + (bs_date.day as i64 - 1);
    Ok(ad_anchor_date() + Duration::days(total_offset))
}

pub fn ad_to_bs(ad_date: NaiveDate) -> Result<BsDate, NcalError> {
    let anchor = ad_anchor_date();
    if ad_date < anchor {
        return Err(NcalError::UnsupportedAdDate);
    }

    let mut remaining_days = (ad_date - anchor).num_days();

    for (year_idx, year_row) in BS_REFERENCE.iter().enumerate() {
        let year_days = year_row[12] as i64;
        if remaining_days >= year_days {
            remaining_days -= year_days;
            continue;
        }

        for (month_idx, month_days) in year_row.iter().take(12).enumerate() {
            let month_days = *month_days as i64;
            if remaining_days >= month_days {
                remaining_days -= month_days;
                continue;
            }

            return Ok(BsDate {
                year: BS_START_YEAR + year_idx as u16,
                month: month_idx as u8 + 1,
                day: remaining_days as u8 + 1,
            });
        }
    }

    Err(NcalError::UnsupportedAdDate)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{
        BS_END_YEAR, BS_REFERENCE, BS_START_YEAR, BsDate, NcalError, ad_to_bs, bs_month_len, bs_to_ad,
        validate_bs_date,
    };

    #[test]
    fn anchor_bs_to_ad_matches_reference() {
        let ad_date = bs_to_ad(BsDate {
            year: 2000,
            month: 1,
            day: 1,
        })
        .unwrap();

        assert_eq!(ad_date, NaiveDate::from_ymd_opt(1943, 4, 14).unwrap());
    }

    #[test]
    fn anchor_ad_to_bs_matches_reference() {
        let bs_date = ad_to_bs(NaiveDate::from_ymd_opt(1943, 4, 14).unwrap()).unwrap();

        assert_eq!(
            bs_date,
            BsDate {
                year: 2000,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn next_day_conversion_works_both_directions() {
        let ad_date = bs_to_ad(BsDate {
            year: 2000,
            month: 1,
            day: 2,
        })
        .unwrap();
        assert_eq!(ad_date, NaiveDate::from_ymd_opt(1943, 4, 15).unwrap());

        let bs_date = ad_to_bs(NaiveDate::from_ymd_opt(1943, 4, 15).unwrap()).unwrap();
        assert_eq!(
            bs_date,
            BsDate {
                year: 2000,
                month: 1,
                day: 2
            }
        );
    }

    #[test]
    fn month_boundary_is_continuous() {
        let last_day_month_1 = bs_to_ad(BsDate {
            year: 2000,
            month: 1,
            day: 30,
        })
        .unwrap();
        let first_day_month_2 = bs_to_ad(BsDate {
            year: 2000,
            month: 2,
            day: 1,
        })
        .unwrap();

        assert_eq!((first_day_month_2 - last_day_month_1).num_days(), 1);
    }

    #[test]
    fn supports_table_end_boundary_roundtrip() {
        let last_month_len = bs_month_len(BS_END_YEAR, 12).unwrap();
        let last_bs = BsDate {
            year: BS_END_YEAR,
            month: 12,
            day: last_month_len,
        };

        let ad = bs_to_ad(last_bs).unwrap();
        let roundtrip = ad_to_bs(ad).unwrap();
        assert_eq!(roundtrip, last_bs);
    }

    #[test]
    fn rejects_out_of_range_year() {
        assert!(matches!(
            bs_to_ad(BsDate {
                year: BS_START_YEAR - 1,
                month: 1,
                day: 1
            }),
            Err(NcalError::InvalidYearRange { .. })
        ));
    }

    #[test]
    fn rejects_invalid_month_and_day() {
        assert!(matches!(
            validate_bs_date(BsDate {
                year: BS_START_YEAR,
                month: 0,
                day: 1
            }),
            Err(NcalError::InvalidMonth)
        ));

        assert!(matches!(
            validate_bs_date(BsDate {
                year: BS_START_YEAR,
                month: 1,
                day: 0
            }),
            Err(NcalError::InvalidDay)
        ));

        assert!(matches!(
            validate_bs_date(BsDate {
                year: BS_START_YEAR,
                month: 1,
                day: 31
            }),
            Err(NcalError::InvalidDay)
        ));
    }

    /// The published tables carry their own year total; a row whose months do not
    /// add up to it is a transcription error, and transcription is the only way
    /// these rows can be wrong.
    #[test]
    fn bs_reference_rows_sum_to_their_total() {
        for (idx, row) in BS_REFERENCE.iter().enumerate() {
            let sum: u16 = row[..12].iter().sum();
            assert_eq!(
                sum,
                row[12],
                "BS {} months sum to {} but the row claims {}",
                BS_START_YEAR + idx as u16,
                sum,
                row[12]
            );
        }
    }

    /// Dates checkable against a published patro. The Kartik 2082 pair is the one
    /// that caught the bad BS 2081-2090 rows: the old table gave Ashwin 2082 thirty
    /// days, putting Kartik 1 a day early and shifting every label after it.
    #[test]
    fn known_bs_ad_pairs() {
        let cases = [
            // (BS y, m, d, AD y, m, d)
            (2082, 1, 1, 2025, 4, 14),   // Nepali New Year 2082
            (2082, 6, 31, 2025, 10, 17), // Ashwin 2082 has 31 days
            (2082, 7, 1, 2025, 10, 18),  // so Kartik 1 is the 18th
            (2082, 7, 3, 2025, 10, 20),  // Laxmi Puja 2082
            (2083, 1, 1, 2026, 4, 14),   // Nepali New Year 2083
            (2083, 5, 16, 2026, 9, 1),
        ];

        for (by, bm, bd, ay, am, ad) in cases {
            let bs = BsDate {
                year: by,
                month: bm,
                day: bd,
            };
            let expected = NaiveDate::from_ymd_opt(ay, am, ad).unwrap();
            assert_eq!(bs_to_ad(bs).unwrap(), expected, "BS {by}-{bm}-{bd} -> AD");
            assert_eq!(ad_to_bs(expected).unwrap(), bs, "AD {expected} -> BS");
        }
    }
}
