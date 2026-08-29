use chrono::{Datelike, Local, NaiveDate};

use crate::config::Config;
use crate::json::{self, DayInfo};
use crate::names;
use crate::{BsDate, NcalError, ad_to_bs, bs_month_len, bs_to_ad};

pub fn run(config: Config) -> Result<(), NcalError> {
    // Nepal is UTC+05:45. Reading the clock in UTC reported yesterday's BS date
    // between local midnight and 05:45 — wrong for a calendar, and fatal for
    // anything that renders today's date continuously.
    let output = render(&config, Local::now().date_naive())?;
    println!("{output}");

    Ok(())
}

/// Render one invocation to a string. `today_ad` is injected rather than read
/// from the clock so callers — and tests — can pin the day.
pub fn render(config: &Config, today_ad: NaiveDate) -> Result<String, NcalError> {
    if config.json {
        return render_json(config, today_ad);
    }
    render_text(config, today_ad)
}

fn render_json(config: &Config, today_ad: NaiveDate) -> Result<String, NcalError> {
    let document = if let Some(raw) = &config.convert {
        json::converted_document(today_ad, DayInfo::from_ad(parse_ad_date(raw)?)?)?
    } else if let Some(raw) = &config.to_ad {
        json::converted_document(today_ad, DayInfo::new(parse_bs_date(raw)?)?)?
    } else if config.today {
        json::today_document(today_ad)?
    } else if let Some(year) = config.year {
        let year = checked_year(year)?;
        match config.month {
            Some(month) => json::month_document(today_ad, year, month)?,
            None => json::year_document(today_ad, year)?,
        }
    } else {
        let today_bs = ad_to_bs(today_ad)?;
        json::month_document(today_ad, today_bs.year, today_bs.month)?
    };

    Ok(json::to_string(&document, !config.compact))
}

fn render_text(config: &Config, today_ad: NaiveDate) -> Result<String, NcalError> {
    if let Some(raw) = &config.convert {
        return Ok(describe(
            &DayInfo::from_ad(parse_ad_date(raw)?)?,
            config.nepali,
        ));
    }
    if let Some(raw) = &config.to_ad {
        return Ok(describe(&DayInfo::new(parse_bs_date(raw)?)?, config.nepali));
    }

    let today_bs = ad_to_bs(today_ad)?;

    if config.today {
        return Ok(describe(&DayInfo::new(today_bs)?, config.nepali));
    }

    if config.year.is_none() {
        return render_bs_month_calendar(today_bs, Some(today_bs.day), config.nepali);
    }

    let year = checked_year(config.year.unwrap())?;
    if config.month.is_none() {
        return render_bs_year_calendar(year, Some(today_bs), config.nepali);
    }

    let bs_date = config.resolved_bs_date()?;
    let highlighted_day = if bs_date.year == today_bs.year && bs_date.month == today_bs.month {
        Some(today_bs.day)
    } else {
        None
    };
    render_bs_month_calendar(bs_date, highlighted_day, config.nepali)
}

fn checked_year(year: u32) -> Result<u16, NcalError> {
    if year < crate::BS_START_YEAR as u32 || year > crate::BS_END_YEAR as u32 {
        return Err(NcalError::InvalidYearRange {
            start: crate::BS_START_YEAR,
            end: crate::BS_END_YEAR,
            got: year,
        });
    }
    Ok(year as u16)
}

fn parse_ad_date(raw: &str) -> Result<NaiveDate, NcalError> {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| NcalError::InvalidDateFormat {
        got: raw.to_string(),
    })
}

fn parse_bs_date(raw: &str) -> Result<BsDate, NcalError> {
    let parts: Vec<&str> = raw.trim().split('-').collect();
    let invalid = || NcalError::InvalidDateFormat {
        got: raw.to_string(),
    };

    if parts.len() != 3 {
        return Err(invalid());
    }

    Ok(BsDate {
        year: parts[0].parse().map_err(|_| invalid())?,
        month: parts[1].parse().map_err(|_| invalid())?,
        day: parts[2].parse().map_err(|_| invalid())?,
    })
}

/// One-line human description of a single day, for `--today` and the converters.
fn describe(info: &DayInfo, nepali: bool) -> String {
    let mut line = if nepali {
        format!(
            "{} {} {} ({}) = {} {}",
            info.bs_day_np,
            info.month_name_np,
            info.bs_year_np,
            info.weekday_name_np,
            info.ad.format("%Y-%m-%d"),
            info.weekday_name,
        )
    } else {
        format!(
            "{} {} {} ({}) = {}",
            info.bs.day,
            info.month_name,
            info.bs.year,
            info.weekday_name,
            info.ad.format("%Y-%m-%d"),
        )
    };

    if !info.festivals.is_empty() {
        let names: Vec<&str> = info
            .festivals
            .iter()
            .map(|f| if nepali { f.np.as_str() } else { f.en.as_str() })
            .collect();
        line.push_str(&format!("  -- {}", names.join(", ")));
    }

    line
}

pub fn render_bs_month_calendar(
    bs_date: BsDate,
    highlighted_day: Option<u8>,
    nepali: bool,
) -> Result<String, NcalError> {
    Ok(render_bs_month_lines(bs_date.year, bs_date.month, highlighted_day, nepali)?.join("\n"))
}

pub fn render_bs_month_lines(
    year: u16,
    month: u8,
    highlighted_day: Option<u8>,
    nepali: bool,
) -> Result<Vec<String>, NcalError> {
    let first_day_ad = bs_to_ad(BsDate {
        year,
        month,
        day: 1,
    })?;
    let month_len = bs_month_len(year, month)?;
    let first_weekday = first_day_ad.weekday().num_days_from_sunday() as usize;

    let title = if nepali {
        format!(
            "{} {}",
            names::bs_month_name_np(month),
            names::to_devanagari(year as u32)
        )
    } else {
        format!("{} {}", names::bs_month_name(month), year)
    };
    let header = center_text(&title, 20);

    let mut cells: Vec<Option<u8>> = vec![None; first_weekday];
    for day in 1..=month_len {
        cells.push(Some(day));
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }

    // The weekday header stays Latin even in Nepali mode: Devanagari weekday
    // abbreviations carry combining vowel signs, so they occupy fewer terminal
    // columns than they do chars and the grid would drift out of alignment.
    let mut lines = vec![header, "Su Mo Tu We Th Fr Sa".to_string()];

    for week in cells.chunks(7) {
        let row = week
            .iter()
            .map(|day| match day {
                Some(d) => {
                    let label = if nepali {
                        names::to_devanagari(*d as u32)
                    } else {
                        d.to_string()
                    };
                    if highlighted_day == Some(*d) {
                        format!("\x1b[7m{:>2}\x1b[0m", label)
                    } else {
                        format!("{:>2}", label)
                    }
                }
                None => "  ".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(row);
    }

    while lines.len() < 8 {
        lines.push("                    ".to_string());
    }

    Ok(lines)
}

pub fn render_bs_year_calendar(
    year: u16,
    highlighted_date: Option<BsDate>,
    nepali: bool,
) -> Result<String, NcalError> {
    let title = if nepali {
        format!("बि.सं. {}", names::to_devanagari(year as u32))
    } else {
        format!("BS {year}")
    };
    let mut lines = vec![center_text(&title, 64), String::new()];

    for block in 0..4 {
        let m1 = block * 3 + 1;
        let m2 = m1 + 1;
        let m3 = m2 + 1;

        let highlighted_in = |month: u8| {
            highlighted_date
                .filter(|d| d.year == year && d.month == month)
                .map(|d| d.day)
        };

        let c1 = render_bs_month_lines(year, m1 as u8, highlighted_in(m1 as u8), nepali)?;
        let c2 = render_bs_month_lines(year, m2 as u8, highlighted_in(m2 as u8), nepali)?;
        let c3 = render_bs_month_lines(year, m3 as u8, highlighted_in(m3 as u8), nepali)?;

        for row in 0..8 {
            lines.push(format!("{:<20}  {:<20}  {:<20}", c1[row], c2[row], c3[row]));
        }

        if block < 3 {
            lines.push(String::new());
        }
    }

    Ok(lines.join("\n"))
}

fn center_text(text: &str, width: usize) -> String {
    // Counted in chars, not bytes: a Devanagari title is multi-byte and would
    // otherwise be treated as far wider than it prints.
    let len = text.chars().count();
    if len >= width {
        return text.to_string();
    }
    let left_pad = (width - len) / 2;
    format!("{}{}", " ".repeat(left_pad), text)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{render, render_bs_month_calendar, render_bs_year_calendar};
    use crate::{BsDate, config::Config};

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 29).unwrap()
    }

    #[test]
    fn renders_cal_like_month_layout() {
        let output = render_bs_month_calendar(
            BsDate {
                year: 2000,
                month: 1,
                day: 1,
            },
            None,
            false,
        )
        .unwrap();

        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("Su Mo Tu We Th Fr Sa"));
        assert!(output.contains("          1  2  3  4"));
        assert!(output.contains("26 27 28 29 30"));
    }

    #[test]
    fn renders_year_calendar_with_all_months() {
        let output = render_bs_year_calendar(2000, None, false).unwrap();

        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("Shrawan 2000"));
        assert!(output.contains("Mangsir 2000"));
        assert!(output.contains("Chaitra 2000"));
        assert!(output.contains("Su Mo Tu We Th Fr Sa"));
    }

    #[test]
    fn no_args_renders_current_bs_month() {
        let config = Config::default();

        let output = render(&config, NaiveDate::from_ymd_opt(1943, 4, 14).unwrap()).unwrap();
        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("\u{1b}[7m 1\u{1b}[0m"));
        assert!(!output.contains("Jestha 2000"));
    }

    #[test]
    fn nepali_mode_uses_devanagari_and_keeps_the_grid_aligned() {
        let output = render_bs_month_calendar(
            BsDate {
                year: 2083,
                month: 5,
                day: 1,
            },
            None,
            true,
        )
        .unwrap();

        assert!(output.contains("भदौ २०८३"));
        assert!(output.contains("Su Mo Tu We Th Fr Sa"));
        for line in output.lines().skip(2) {
            assert_eq!(line.chars().count(), 20, "row is not 20 columns: {line:?}");
        }
    }

    #[test]
    fn today_flag_describes_one_day() {
        let config = Config {
            today: true,
            ..Config::default()
        };

        let output = render(&config, today()).unwrap();
        assert!(output.contains("13 Bhadra 2083"));
        assert!(output.contains("Saturday"));
        assert!(output.contains("2026-08-29"));
    }

    #[test]
    fn convert_flag_maps_ad_to_bs() {
        let config = Config {
            convert: Some("2026-08-29".to_string()),
            ..Config::default()
        };

        assert!(render(&config, today()).unwrap().contains("13 Bhadra 2083"));
    }

    #[test]
    fn to_ad_flag_maps_bs_to_ad() {
        let config = Config {
            to_ad: Some("2083-05-13".to_string()),
            ..Config::default()
        };

        assert!(render(&config, today()).unwrap().contains("2026-08-29"));
    }

    #[test]
    fn a_malformed_date_is_rejected() {
        let config = Config {
            convert: Some("29/08/2026".to_string()),
            ..Config::default()
        };

        assert!(matches!(
            render(&config, today()),
            Err(crate::NcalError::InvalidDateFormat { .. })
        ));
    }

    #[test]
    fn festivals_show_up_in_the_one_line_description() {
        let config = Config {
            to_ad: Some("2083-01-01".to_string()),
            ..Config::default()
        };

        assert!(render(&config, today()).unwrap().contains("New Year"));
    }

    #[test]
    fn json_mode_emits_parseable_json() {
        let config = Config {
            json: true,
            ..Config::default()
        };

        let output = render(&config, today()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["today"]["ad"], "2026-08-29");
        assert_eq!(value["month"]["year"], 2083);
    }

    #[test]
    fn json_year_view_emits_twelve_months() {
        let config = Config {
            json: true,
            year: Some(2083),
            ..Config::default()
        };

        let output = render(&config, today()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value["months"].as_array().unwrap().len(), 12);
    }
}
