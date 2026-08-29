pub mod app;
pub mod cli;
pub mod config;
pub mod festivals;
pub mod grid;
pub mod json;
pub mod names;

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Used for offsetting the BS date using the AD date
// The first 12 elements represent the number of days in each month, and the 13th element represents the total number of days in that year (365 or 366).
#[allow(unused)]
pub const BS_REFERENCE: [[u16; 13]; 91] = [
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 32, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 32, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [30, 32, 31, 32, 31, 31, 29, 30, 29, 30, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 29, 31, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 29, 30, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 30, 29, 31, 366],
    [31, 31, 31, 32, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 29, 30, 29, 30, 30, 365],
    [31, 32, 31, 32, 31, 30, 30, 30, 29, 29, 30, 30, 365],
    [31, 31, 32, 32, 31, 30, 30, 30, 29, 30, 30, 30, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 30, 30, 365],
    [31, 31, 32, 31, 31, 30, 30, 30, 29, 30, 30, 30, 365],
    [31, 31, 32, 31, 31, 30, 30, 30, 29, 30, 30, 30, 365],
    [31, 32, 31, 32, 30, 31, 30, 30, 29, 30, 30, 30, 366],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 30, 30, 365],
    [31, 31, 32, 31, 31, 31, 30, 30, 29, 30, 30, 30, 366],
    [30, 31, 32, 32, 30, 31, 30, 30, 29, 30, 30, 30, 365],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 30, 30, 365],
    [30, 32, 31, 32, 31, 30, 30, 30, 29, 30, 30, 30, 365],
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
        BS_END_YEAR, BS_START_YEAR, BsDate, NcalError, ad_to_bs, bs_month_len, bs_to_ad,
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
}
