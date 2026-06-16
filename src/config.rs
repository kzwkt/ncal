use crate::{BsDate, NcalError, cli::Cli, validate_bs_date};

#[derive(Debug, Clone)]
pub struct Config {
    pub day: Option<u8>,
    pub month: Option<u8>,
    pub year: Option<u32>,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self, NcalError> {
        match cli.parse_args() {
            Ok((day, month, year)) => Ok(Self { day, month, year }),
            Err(e) => Err(e),
        }
    }

    pub fn validate(&self) -> Result<(), NcalError> {
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
        let config = Config {
            day: None,
            month: None,
            year: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_month_without_year() {
        let config = Config {
            day: None,
            month: Some(1),
            year: None,
        };

        assert!(matches!(config.validate(), Err(NcalError::MissingYear)));
    }
}
