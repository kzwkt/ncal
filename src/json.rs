//! The `--json` document.
//!
//! This is the machine-readable face of `ncal` and the entire API surface a GUI
//! needs: today, one month laid out in weeks, and that month's festivals. Treat
//! the field names here as a contract — renaming one breaks every consumer.

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::festivals::{self, Festival};
use crate::grid::{self, MonthGrid};
use crate::names;
use crate::panchanga::Reading;
use crate::{BS_END_YEAR, BS_START_YEAR, BsDate, NcalError, ad_to_bs, bs_to_ad};

/// What this build of `ncal` can answer questions about.
#[derive(Debug, Clone, Serialize)]
pub struct Range {
    pub bs_start: u16,
    pub bs_end: u16,
    /// Where tithi-based festivals come from. `"computed"` means ncal derives
    /// them from the lunar rules rather than reading a per-year table, so a
    /// consumer need not warn that Dashain or Tihar might be absent.
    pub festival_source: &'static str,
    /// Inclusive `[first, last]` BS years for which festivals are available.
    pub festival_range: [u16; 2],
    /// Inclusive `[first, last]` BS years carrying hand-verified overrides of the
    /// computed dates, or `null` when there are none.
    pub override_years: Option<[u16; 2]>,
}

impl Range {
    fn current() -> Self {
        Self {
            bs_start: BS_START_YEAR,
            bs_end: BS_END_YEAR,
            festival_source: "computed",
            festival_range: {
                let (first, last) = festivals::festival_coverage();
                [first, last]
            },
            override_years: festivals::override_years().map(|(a, b)| [a, b]),
        }
    }
}

/// One fully-described day, in both calendars and both scripts.
#[derive(Debug, Clone, Serialize)]
pub struct DayInfo {
    pub bs: BsDate,
    pub ad: NaiveDate,
    /// Sunday = 0.
    pub weekday: u8,
    pub month_name: &'static str,
    pub month_name_np: &'static str,
    pub weekday_name: &'static str,
    pub weekday_name_np: &'static str,
    pub bs_day_np: String,
    pub bs_year_np: String,
    pub is_holiday: bool,
    pub festivals: Vec<Festival>,
    pub panchanga: Reading,
}

impl DayInfo {
    pub fn new(bs: BsDate) -> Result<Self, NcalError> {
        let ad = bs_to_ad(bs)?;
        let weekday = ad.weekday().num_days_from_sunday() as u8;
        let festivals = festivals::festivals_on(bs, ad);

        Ok(Self {
            bs,
            ad,
            weekday,
            month_name: names::bs_month_name(bs.month),
            month_name_np: names::bs_month_name_np(bs.month),
            weekday_name: names::weekday_name(weekday),
            weekday_name_np: names::weekday_name_np(weekday),
            bs_day_np: names::to_devanagari(bs.day as u32),
            bs_year_np: names::to_devanagari(bs.year as u32),
            is_holiday: weekday == 6 || festivals.iter().any(|f| f.holiday),
            festivals,
            panchanga: Reading::for_date(ad),
        })
    }

    pub fn from_ad(ad: NaiveDate) -> Result<Self, NcalError> {
        Self::new(ad_to_bs(ad)?)
    }
}

/// A festival with the BS day it falls on, for the month's festival list.
#[derive(Debug, Clone, Serialize)]
pub struct DatedFestival {
    pub bs_day: u8,
    pub bs_day_np: String,
    #[serde(flatten)]
    pub festival: Festival,
}

/// The top-level `--json` document. Optional sections are omitted rather than
/// null so `--today` stays small enough to run on every bar repaint.
#[derive(Debug, Clone, Serialize)]
pub struct Document {
    /// Version of `ncal` that produced this, so a GUI can refuse an old binary.
    pub ncal: &'static str,
    pub range: Range,
    pub today: DayInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<MonthGrid>,
    /// All twelve grids, for the whole-year view.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub months: Option<Vec<MonthGrid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub festivals: Option<Vec<DatedFestival>>,
    /// Present only for `--convert` / `--to-ad`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub converted: Option<DayInfo>,
}

/// Just today: the cheap path a status bar polls.
pub fn today_document(today_ad: NaiveDate) -> Result<Document, NcalError> {
    Ok(Document {
        ncal: env!("CARGO_PKG_VERSION"),
        range: Range::current(),
        today: DayInfo::from_ad(today_ad)?,
        month: None,
        months: None,
        festivals: None,
        converted: None,
    })
}

/// Today plus one full month grid and that month's festivals.
pub fn month_document(today_ad: NaiveDate, year: u16, month: u8) -> Result<Document, NcalError> {
    let grid = grid::month_grid(year, month, today_ad)?;
    let festivals = festivals::festivals_in_month(year, month)
        .into_iter()
        .map(|(bs_day, festival)| DatedFestival {
            bs_day,
            bs_day_np: names::to_devanagari(bs_day as u32),
            festival,
        })
        .collect();

    Ok(Document {
        ncal: env!("CARGO_PKG_VERSION"),
        range: Range::current(),
        today: DayInfo::from_ad(today_ad)?,
        month: Some(grid),
        months: None,
        festivals: Some(festivals),
        converted: None,
    })
}

/// Today plus all twelve months of one BS year, mirroring `ncal <year>`.
pub fn year_document(today_ad: NaiveDate, year: u16) -> Result<Document, NcalError> {
    let mut months = Vec::with_capacity(12);
    let mut festivals = Vec::new();
    for month in 1..=12u8 {
        months.push(grid::month_grid(year, month, today_ad)?);
        festivals.extend(festivals::festivals_in_month(year, month).into_iter().map(
            |(bs_day, festival)| DatedFestival {
                bs_day,
                bs_day_np: names::to_devanagari(bs_day as u32),
                festival,
            },
        ));
    }

    Ok(Document {
        ncal: env!("CARGO_PKG_VERSION"),
        range: Range::current(),
        today: DayInfo::from_ad(today_ad)?,
        month: None,
        months: Some(months),
        festivals: Some(festivals),
        converted: None,
    })
}

/// Today plus a single converted date.
pub fn converted_document(today_ad: NaiveDate, converted: DayInfo) -> Result<Document, NcalError> {
    Ok(Document {
        ncal: env!("CARGO_PKG_VERSION"),
        range: Range::current(),
        today: DayInfo::from_ad(today_ad)?,
        month: None,
        months: None,
        festivals: None,
        converted: Some(converted),
    })
}

/// Serialize a document. Pretty-printing costs nothing at this size and makes
/// `ncal --json | less` usable by hand.
pub fn to_string(document: &Document, pretty: bool) -> String {
    let rendered = if pretty {
        serde_json::to_string_pretty(document)
    } else {
        serde_json::to_string(document)
    };
    rendered.expect("document is plain data and must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 29).unwrap()
    }

    fn parse(document: &Document) -> serde_json::Value {
        serde_json::from_str(&to_string(document, false)).unwrap()
    }

    #[test]
    fn today_document_carries_the_expected_keys() {
        let value = parse(&today_document(today()).unwrap());

        assert!(value["ncal"].is_string());
        assert_eq!(value["range"]["bs_start"], crate::BS_START_YEAR);
        assert_eq!(value["range"]["bs_end"], crate::BS_END_YEAR);
        assert!(value["today"]["bs"]["year"].is_number());
        assert_eq!(value["today"]["ad"], "2026-08-29");
        assert!(value["today"]["month_name_np"].is_string());
        // The cheap path must not drag a whole month grid along.
        assert!(value.get("month").is_none());
        assert!(value.get("converted").is_none());
    }

    #[test]
    fn today_resolves_to_the_right_bs_date() {
        let document = today_document(today()).unwrap();
        assert_eq!(
            document.today.bs,
            BsDate {
                year: 2083,
                month: 5,
                day: 13
            }
        );
        assert_eq!(document.today.month_name, "Bhadra");
        assert_eq!(document.today.month_name_np, "भदौ");
        assert_eq!(document.today.bs_day_np, "१३");
        assert_eq!(document.today.bs_year_np, "२०८३");
    }

    #[test]
    fn month_document_has_a_grid_of_seven_wide_weeks() {
        let value = parse(&month_document(today(), 2083, 5).unwrap());

        assert_eq!(value["month"]["year"], 2083);
        assert_eq!(value["month"]["name"], "Bhadra");
        assert_eq!(value["month"]["days"], 31);
        assert_eq!(value["month"]["year_np"], "२०८३");
        let weeks = value["month"]["weeks"].as_array().unwrap();
        assert!(weeks.iter().all(|w| w.as_array().unwrap().len() == 7));
        assert!(value["festivals"].is_array());
        let new_year = parse(&month_document(today(), 2083, 1).unwrap());
        assert_eq!(new_year["festivals"][0]["bs_day_np"], "१");
    }

    #[test]
    fn month_document_marks_today_inside_the_grid() {
        let value = parse(&month_document(today(), 2083, 5).unwrap());
        let marked: Vec<i64> = value["month"]["weeks"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|week| week.as_array().unwrap())
            .filter(|cell| !cell.is_null() && cell["is_today"] == true)
            .map(|cell| cell["bs_day"].as_i64().unwrap())
            .collect();
        assert_eq!(marked, vec![13]);
    }

    #[test]
    fn converted_document_reports_both_calendars() {
        let converted = DayInfo::from_ad(NaiveDate::from_ymd_opt(2026, 4, 14).unwrap()).unwrap();
        let value = parse(&converted_document(today(), converted).unwrap());

        assert_eq!(value["converted"]["ad"], "2026-04-14");
        assert!(value["converted"]["bs"]["year"].is_number());
        assert!(value["converted"]["weekday_name_np"].is_string());
    }

    #[test]
    fn new_year_is_a_holiday_in_the_document() {
        let info = DayInfo::new(BsDate {
            year: 2083,
            month: 1,
            day: 1,
        })
        .unwrap();
        assert!(info.is_holiday);
        assert!(info.festivals.iter().any(|f| f.en.contains("New Year")));
    }

    #[test]
    fn out_of_range_month_is_an_error_not_a_panic() {
        assert!(month_document(today(), 2083, 13).is_err());
        assert!(month_document(today(), 3000, 1).is_err());
    }
}
