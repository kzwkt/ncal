use chrono::{Datelike, NaiveDate, Utc};

use crate::{BsDate, NcalError, ad_to_bs, bs_month_len, bs_to_ad};

use crate::config::Config;

pub fn run(config: Config) -> Result<(), NcalError> {
    let output = render_output(&config, Utc::now().date_naive())?;
    println!("{output}");

    Ok(())
}

fn render_output(config: &Config, today_ad: NaiveDate) -> Result<String, NcalError> {
    let today_bs = ad_to_bs(today_ad)?;

    if config.year.is_none() {
        let output = render_bs_month_calendar(today_bs, Some(today_bs.day))?;
        return Ok(output);
    }

    let year = config.year.unwrap() as u16;
    if config.month.is_none() {
        return render_bs_year_calendar(year, Some(today_bs));
    }

    let bs_date = config.resolved_bs_date()?;
    let highlighted_day = if bs_date.year == today_bs.year && bs_date.month == today_bs.month {
        Some(today_bs.day)
    } else {
        None
    };
    render_bs_month_calendar(bs_date, highlighted_day)
}

fn render_bs_month_calendar(bs_date: BsDate, highlighted_day: Option<u8>) -> Result<String, NcalError> {
    Ok(render_bs_month_lines(bs_date.year, bs_date.month, highlighted_day)?.join("\n"))
}

fn render_bs_month_lines(year: u16, month: u8, highlighted_day: Option<u8>) -> Result<Vec<String>, NcalError> {
    let first_day_ad = bs_to_ad(BsDate {
        year,
        month,
        day: 1,
    })?;
    let month_len = bs_month_len(year, month)?;
    let first_weekday = first_day_ad.weekday().num_days_from_sunday() as usize;

    let title = format!("{} {}", bs_month_name(month), year);
    let header = center_text(&title, 20);

    let mut cells: Vec<Option<u8>> = vec![None; first_weekday];
    for day in 1..=month_len {
        cells.push(Some(day));
    }
    while cells.len() % 7 != 0 {
        cells.push(None);
    }

    let mut lines = vec![header, "Su Mo Tu We Th Fr Sa".to_string()];

    for week in cells.chunks(7) {
        let row = week
            .iter()
            .map(|day| match day {
                Some(d) if highlighted_day == Some(*d) => format!("\x1b[7m{:>2}\x1b[0m", d),
                Some(d) => format!("{:>2}", d),
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

fn render_bs_year_calendar(year: u16, highlighted_date: Option<BsDate>) -> Result<String, NcalError> {
    let mut lines = vec![center_text(&format!("BS {year}"), 64), String::new()];

    for block in 0..4 {
        let m1 = block * 3 + 1;
        let m2 = m1 + 1;
        let m3 = m2 + 1;

        let h1 = highlighted_date
            .filter(|d| d.year == year && d.month == m1 as u8)
            .map(|d| d.day);
        let h2 = highlighted_date
            .filter(|d| d.year == year && d.month == m2 as u8)
            .map(|d| d.day);
        let h3 = highlighted_date
            .filter(|d| d.year == year && d.month == m3 as u8)
            .map(|d| d.day);

        let c1 = render_bs_month_lines(year, m1 as u8, h1)?;
        let c2 = render_bs_month_lines(year, m2 as u8, h2)?;
        let c3 = render_bs_month_lines(year, m3 as u8, h3)?;

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
    if text.len() >= width {
        return text.to_string();
    }
    let left_pad = (width - text.len()) / 2;
    format!("{}{}", " ".repeat(left_pad), text)
}

fn bs_month_name(month: u8) -> &'static str {
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{render_bs_month_calendar, render_bs_year_calendar, render_output};
    use crate::{BsDate, config::Config};

    #[test]
    fn renders_cal_like_month_layout() {
        let output = render_bs_month_calendar(BsDate {
            year: 2000,
            month: 1,
            day: 1,
        }, None)
        .unwrap();

        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("Su Mo Tu We Th Fr Sa"));
        assert!(output.contains("          1  2  3  4"));
        assert!(output.contains("26 27 28 29 30"));
    }

    #[test]
    fn renders_year_calendar_with_all_months() {
        let output = render_bs_year_calendar(2000, None).unwrap();

        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("Shrawan 2000"));
        assert!(output.contains("Mangsir 2000"));
        assert!(output.contains("Chaitra 2000"));
        assert!(output.contains("Su Mo Tu We Th Fr Sa"));
    }

    #[test]
    fn no_args_renders_current_bs_month() {
        let config = Config {
            day: None,
            month: None,
            year: None,
        };

        let output = render_output(&config, NaiveDate::from_ymd_opt(1943, 4, 14).unwrap()).unwrap();
        assert!(output.contains("Baisakh 2000"));
        assert!(output.contains("\u{1b}[7m 1\u{1b}[0m"));
        assert!(!output.contains("Jestha 2000"));
    }
}
