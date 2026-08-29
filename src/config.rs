use crate::{BsDate, NcalError, cli::Cli, validate_bs_date};

/// The parsed intent of one invocation: which month to show, and how.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub day: Option<u8>,
    pub month: Option<u8>,
    pub year: Option<u32>,
    pub json: bool,
    pub today: bool,
    pub convert: Option<String>,
    pub to_ad: Option<String>,
    pub nepali: bool,
    pub compact: bool,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self, NcalError> {
        let (day, month, year) = cli.parse_args()?;
        Ok(Self {
            day,
            month,
            year,
            json: cli.json,
            today: cli.today,
            convert: cli.convert.clone(),
            to_ad: cli.to_ad.clone(),
            nepali: cli.nepali,
            compact: cli.compact,
        })
    }

    /// True when the invocation asks a one-off question rather than for a
    /// calendar, in which case the positional month/year are irrelevant.
    pub fn is_query(&self) -> bool {
        self.today || self.convert.is_some() || self.to_ad.is_some()
    }

    pub fn validate(&self) -> Result<(), NcalError> {
        if self.is_query() {
            return Ok(());
        }

        if self.year.is_none() {
            if self.month.is_some() || self.day.is_some() {
                return Err(NcalError::MissingYear);
            }
            return Ok(());
        }

        let bs_date = self.resolved_bs_date()?;
        validate_bs_date(bs_date)?;
        Ok(())
    }

    pub fn resolved_bs_date(&self) -> Result<BsDate, NcalError> {
        let year = self.year.ok_or(NcalError::MissingYear)?;
        let month = self.month.unwrap_or(1);
        let day = self.day.unwrap_or(1);

        if year < crate::BS_START_YEAR as u32 || year > crate::BS_END_YEAR as u32 {
            return Err(NcalError::InvalidYearRange {
                start: crate::BS_START_YEAR,
                end: crate::BS_END_YEAR,
                got: year,
            });
        }

        Ok(BsDate {
            year: year as u16,
            month,
            day,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::NcalError;

    #[test]
    fn validate_allows_empty_config_for_current_date_mode() {
        let config = Config::default();

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_month_without_year() {
        let config = Config {
            month: Some(1),
            ..Config::default()
        };

        assert!(matches!(config.validate(), Err(NcalError::MissingYear)));
    }

    #[test]
    fn queries_skip_calendar_validation() {
        // `--convert` has no month/year of its own; requiring one would be wrong.
        let config = Config {
            convert: Some("2026-08-29".to_string()),
            ..Config::default()
        };

        assert!(config.is_query());
        assert!(config.validate().is_ok());
    }
}
