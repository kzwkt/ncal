use clap::Parser;

use crate::NcalError;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// [[[day] month] year]]
    #[arg(value_parser = clap::value_parser!(u32))]
    args: Vec<u32>,

    /// Emit machine-readable JSON instead of the calendar grid
    #[arg(long)]
    pub json: bool,

    /// Report only today's date; with --json this skips the month grid
    #[arg(long)]
    pub today: bool,

    /// Convert a Gregorian date to Bikram Sambat
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub convert: Option<String>,

    /// Convert a Bikram Sambat date to Gregorian
    #[arg(long = "to-ad", value_name = "YYYY-MM-DD")]
    pub to_ad: Option<String>,

    /// Use Devanagari month names and numerals in the calendar output
    #[arg(long)]
    pub nepali: bool,

    /// Print JSON on one line instead of indented
    #[arg(long)]
    pub compact: bool,
}

#[allow(unused)]
impl Cli {
    pub fn parse_args(&self) -> Result<(Option<u8>, Option<u8>, Option<u32>), NcalError> {
        let mut day: Option<u8> = None;
        let mut month: Option<u8> = None;
        let mut year: Option<u32> = None;

        match self.args.len() {
            0 => {
                return Ok((None, None, None));
            }

            1 => {
                year = Some(self.args.get(0).cloned().unwrap());
            }
            2 => {
                month = Some(self.args.get(0).cloned().unwrap() as u8);
                year = Some(self.args.get(1).cloned().unwrap());
            }
            3 => {
                day = Some(self.args.get(0).cloned().unwrap() as u8);
                month = Some(self.args.get(1).cloned().unwrap() as u8);
                year = Some(self.args.get(2).cloned().unwrap());
            }
            _ => {
                return Err(NcalError::InvalidArgumentLength {
                    arg: format!("{:?}", self.args.len()),
                });
            }
        }

        Ok((day, month, year))
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn positional_arguments_still_parse_as_before() {
        let cli = Cli::try_parse_from(["ncal", "15", "3", "2083"]).unwrap();
        assert_eq!(cli.parse_args().unwrap(), (Some(15), Some(3), Some(2083)));

        let cli = Cli::try_parse_from(["ncal", "2083"]).unwrap();
        assert_eq!(cli.parse_args().unwrap(), (None, None, Some(2083)));

        let cli = Cli::try_parse_from(["ncal"]).unwrap();
        assert_eq!(cli.parse_args().unwrap(), (None, None, None));
    }

    #[test]
    fn flags_coexist_with_positional_arguments() {
        let cli = Cli::try_parse_from(["ncal", "--json", "3", "2083"]).unwrap();
        assert!(cli.json);
        assert_eq!(cli.parse_args().unwrap(), (None, Some(3), Some(2083)));
    }

    #[test]
    fn conversion_flags_parse() {
        let cli = Cli::try_parse_from(["ncal", "--convert", "2026-08-29"]).unwrap();
        assert_eq!(cli.convert.as_deref(), Some("2026-08-29"));

        let cli = Cli::try_parse_from(["ncal", "--to-ad", "2083-05-13"]).unwrap();
        assert_eq!(cli.to_ad.as_deref(), Some("2083-05-13"));
    }
}
