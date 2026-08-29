//! The calendar month as data rather than as text.
//!
//! `render_bs_month_lines` in `app` already computes this shape internally in
//! order to throw it away into strings. `month_grid` is the same computation with
//! the formatting removed, so a GUI can lay the month out itself.

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::festivals::{self, Festival};
use crate::names;
use crate::{BsDate, NcalError, bs_month_len, bs_to_ad};

/// One day of a BS month, with the Gregorian date it maps to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DayCell {
    pub bs_day: u8,
    /// `bs_day` in Devanagari digits, so the UI does not have to reimplement it.
    pub bs_day_np: String,
    pub ad: NaiveDate,
    pub ad_day: u32,
    /// Abbreviated Gregorian month, for the small corner label: `Aug`.
    pub ad_month_short: String,
    /// Sunday = 0, matching `chrono::Weekday::num_days_from_sunday`.
    pub weekday: u8,
    pub is_today: bool,
    /// Saturday (the Nepali weekend) or a festival flagged as a public holiday.
    pub is_holiday: bool,
    pub festivals: Vec<Festival>,
}

/// A BS month laid out in weeks, Sunday-first, padded with `None` at both ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthGrid {
    pub year: u16,
    pub month: u8,
    pub days: u8,
    pub name: &'static str,
    pub name_np: &'static str,
    /// Gregorian date of BS day 1.
    pub start_ad: NaiveDate,
    /// Gregorian date of the last BS day of the month.
    pub end_ad: NaiveDate,
    /// Each inner vector is exactly 7 long.
    pub weeks: Vec<Vec<Option<DayCell>>>,
}

const SATURDAY: u8 = 6;

const AD_MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn day_cell(bs: BsDate, ad: NaiveDate, today: NaiveDate) -> DayCell {
    let weekday = ad.weekday().num_days_from_sunday() as u8;
    let festivals = festivals::festivals_on(bs, ad);

    DayCell {
        bs_day: bs.day,
        bs_day_np: names::to_devanagari(bs.day as u32),
        ad,
        ad_day: ad.day(),
        ad_month_short: AD_MONTH_SHORT[(ad.month() - 1) as usize].to_string(),
        weekday,
        is_today: ad == today,
        is_holiday: weekday == SATURDAY || festivals.iter().any(|f| f.holiday),
        festivals,
    }
}

/// Build the grid for one BS month. `today` is passed in rather than read from
/// the clock so this stays a pure function and stays testable.
pub fn month_grid(year: u16, month: u8, today: NaiveDate) -> Result<MonthGrid, NcalError> {
    let month_len = bs_month_len(year, month)?;
    let start_ad = bs_to_ad(BsDate {
        year,
        month,
        day: 1,
    })?;
    let end_ad = bs_to_ad(BsDate {
        year,
        month,
        day: month_len,
    })?;

    let leading = start_ad.weekday().num_days_from_sunday() as usize;
    let mut cells: Vec<Option<DayCell>> = vec![None; leading];

    for day in 1..=month_len {
        let bs = BsDate { year, month, day };
        let ad = bs_to_ad(bs)?;
        cells.push(Some(day_cell(bs, ad, today)));
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }

    Ok(MonthGrid {
        year,
        month,
        days: month_len,
        name: names::bs_month_name(month),
        name_np: names::bs_month_name_np(month),
        start_ad,
        end_ad,
        weeks: cells.chunks(7).map(|week| week.to_vec()).collect(),
    })
}

/// Step a BS year/month by `delta` months, clamped to the supported year range.
/// Returns `None` when the step would leave the table.
pub fn step_month(year: u16, month: u8, delta: i32) -> Option<(u16, u8)> {
    let absolute = (year as i32) * 12 + (month as i32 - 1) + delta;
    let new_year = absolute.div_euclid(12);
    let new_month = absolute.rem_euclid(12) + 1;

    if new_year < crate::BS_START_YEAR as i32 || new_year > crate::BS_END_YEAR as i32 {
        return None;
    }
    Some((new_year as u16, new_month as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn some_day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
    }

    #[test]
    fn grid_holds_exactly_the_months_days() {
        let grid = month_grid(2083, 5, some_day()).unwrap();
        let days: Vec<u8> = grid
            .weeks
            .iter()
            .flatten()
            .filter_map(|c| c.as_ref().map(|c| c.bs_day))
            .collect();

        assert_eq!(days.len(), grid.days as usize);
        assert_eq!(days.first(), Some(&1));
        assert_eq!(days.last(), Some(&grid.days));
        assert!(days.windows(2).all(|w| w[1] == w[0] + 1));
    }

    #[test]
    fn every_week_is_seven_wide() {
        for month in 1..=12u8 {
            let grid = month_grid(2083, month, some_day()).unwrap();
            assert!(grid.weeks.iter().all(|week| week.len() == 7));
        }
    }

    #[test]
    fn leading_padding_matches_the_first_weekday() {
        let grid = month_grid(2083, 5, some_day()).unwrap();
        let leading = grid.weeks[0].iter().take_while(|c| c.is_none()).count();
        assert_eq!(
            leading,
            grid.start_ad.weekday().num_days_from_sunday() as usize
        );
    }

    #[test]
    fn day_one_lines_up_with_bs_to_ad() {
        let grid = month_grid(2083, 5, some_day()).unwrap();
        let first = grid
            .weeks
            .iter()
            .flatten()
            .flatten()
            .find(|c| c.bs_day == 1)
            .unwrap();
        assert_eq!(
            first.ad,
            bs_to_ad(BsDate {
                year: 2083,
                month: 5,
                day: 1
            })
            .unwrap()
        );
        assert_eq!(first.bs_day_np, "१");
    }

    #[test]
    fn today_is_marked_exactly_once() {
        let today = bs_to_ad(BsDate {
            year: 2083,
            month: 5,
            day: 13,
        })
        .unwrap();
        let grid = month_grid(2083, 5, today).unwrap();
        let marked: Vec<u8> = grid
            .weeks
            .iter()
            .flatten()
            .flatten()
            .filter(|c| c.is_today)
            .map(|c| c.bs_day)
            .collect();
        assert_eq!(marked, vec![13]);
    }

    #[test]
    fn saturdays_are_holidays() {
        let grid = month_grid(2083, 5, some_day()).unwrap();
        for cell in grid.weeks.iter().flatten().flatten() {
            if cell.weekday == SATURDAY {
                assert!(cell.is_holiday, "BS day {} is a Saturday", cell.bs_day);
            }
        }
    }

    #[test]
    fn new_year_day_carries_its_festival() {
        let grid = month_grid(2083, 1, some_day()).unwrap();
        let first = grid
            .weeks
            .iter()
            .flatten()
            .flatten()
            .find(|c| c.bs_day == 1)
            .unwrap();
        assert!(first.festivals.iter().any(|f| f.en.contains("New Year")));
        assert!(first.is_holiday);
    }

    #[test]
    fn stepping_months_wraps_the_year() {
        assert_eq!(step_month(2083, 12, 1), Some((2084, 1)));
        assert_eq!(step_month(2083, 1, -1), Some((2082, 12)));
        assert_eq!(step_month(2083, 5, 12), Some((2084, 5)));
    }

    #[test]
    fn stepping_off_the_table_is_none() {
        assert_eq!(step_month(crate::BS_END_YEAR, 12, 1), None);
        assert_eq!(step_month(crate::BS_START_YEAR, 1, -1), None);
    }

    #[test]
    fn every_supported_month_builds() {
        for year in [crate::BS_START_YEAR, 2083, crate::BS_END_YEAR] {
            for month in 1..=12u8 {
                assert!(
                    month_grid(year, month, some_day()).is_ok(),
                    "BS {year}-{month}"
                );
            }
        }
    }
}
