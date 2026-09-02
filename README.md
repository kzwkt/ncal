# ncal

`ncal` is a terminal Bikram Sambat (BS) calendar tool, inspired by Unix `cal`.



It supports:
- AD <-> BS date conversion, in both directions
- current-day highlighting in calendar output
- Devanagari month names, weekday names and numerals
- festivals and public holidays, computed from the ephemeris rather than tabulated
- the day's panchanga: tithi, paksha, and sunrise/sunset for Kathmandu
- JSON output, so other programs can build on it

## Build ,run and install

Requirements:
- Rust toolchain (stable)
- Cargo

Build:

```bash
cargo build
```

Run:

```bash
cargo run -- [args]
```

Install:

```bash
cargo install ncal
```

## Usage

```text
ncal [OPTIONS] [[[day] month] year]
```

The positional arguments are interpreted as:
- `ncal` -> show current BS month (derived from current AD date)
- `ncal <year>` -> show full BS year calendar (12 months)
- `ncal <month> <year>` -> show one BS month
- `ncal <day> <month> <year>` -> show one BS month (day is accepted and validated)

Examples:

```bash
ncal
ncal 2083
ncal 3 2083
ncal 15 3 2083
```

### Options

| Flag | Effect |
| --- | --- |
| `--today` | print just today's date, in both calendars |
| `--convert YYYY-MM-DD` | convert a Gregorian date to Bikram Sambat |
| `--to-ad YYYY-MM-DD` | convert a Bikram Sambat date to Gregorian |
| `--nepali` | Devanagari month names and numerals |
| `--json` | emit JSON instead of the calendar grid |
| `--compact` | with `--json`, print on one line |

```bash
ncal --today
ncal --today --nepali          # १३ भदौ २०८३ (शनिबार) = 2026-08-29 Saturday
ncal --convert 2026-04-14      # 1 Baisakh 2083 (Tuesday) -- Nepali New Year
ncal --to-ad 2083-01-01
ncal --nepali 5 2083
```

## JSON output

`--json` is the machine-readable interface. It always carries `today`; it adds `month`
(one grid) or `months` (a whole year) depending on the positional arguments, and
`converted` when a conversion flag is used.

```bash
ncal --json --today            # cheap: today only, no grid
ncal --json 5 2083             # one month, laid out in Sunday-first weeks
ncal --json 2083               # all twelve months
ncal --json --convert 2026-04-14
```

```jsonc
{
  "ncal": "0.3.0",
  "range": {
    "bs_start": 2000, "bs_end": 2099,
    "festival_source": "computed",   // festivals are derived, not tabulated
    "festival_range": [2000, 2099],
    "override_years": null           // BS years with a hand-verified correction
  },
  "today": {
    "bs": { "year": 2083, "month": 5, "day": 13 },
    "ad": "2026-08-29", "weekday": 6,
    "month_name": "Bhadra", "month_name_np": "भदौ",
    "weekday_name": "Saturday", "weekday_name_np": "शनिबार",
    "bs_day_np": "१३", "bs_year_np": "२०८३",
    "is_holiday": true, "festivals": [],
    "panchanga": {
      "tithi": 12, "tithi_name": "Dwadashi", "tithi_name_np": "द्वादशी",
      "paksha": "Shukla", "paksha_np": "शुक्ल पक्ष",
      "tithi_ends": "14:22",           // Nepal time, or null if it outlasts the day
      "sunrise": "05:43", "sunset": "18:25"
    }
  },
  "month": {
    "year": 2083, "month": 5, "days": 31, "name": "Bhadra", "name_np": "भदौ",
    "year_np": "२०८३",
    "start_ad": "2026-08-17", "end_ad": "2026-09-16",
    "weeks": [ [ null, null, { "bs_day": 1, "bs_day_np": "१", "ad": "2026-08-17",
                               "ad_day": 17, "ad_month_short": "Aug", "weekday": 2,
                               "is_today": false, "is_holiday": false,
                               "festivals": [], "panchanga": { /* as above */ } } ] ]
  },
  "festivals": [ { "bs_day": 1, "en": "Nepali New Year", "np": "नयाँ वर्ष", "holiday": true } ]
}
```

Each week is exactly seven entries, Sunday first, padded with `null` at both ends.
Weekdays count from Sunday = 0. Treat these field names as a contract.

## Festivals

Festival and panchanga data are embedded with `include_str!`, so there is no runtime file
dependency and no network access.

**Tithi-based festivals are computed, not tabulated.** Dashain, Tihar, Teej, Shivaratri,
Holi and the rest are stated in `src/data/rules.json` as the rule they actually are —
which lunar month, which paksha, which tithi, and at which kāla the tithi has to be
running — and `src/panchanga.rs` resolves that to a date for any year in range. There is
no per-year table to maintain and nothing that goes stale.

That works because a tithi is a *difference* between the moon's and the sun's longitudes.
Whatever ayanamsa you would add to both cancels exactly, so no ephemeris calibration is
needed and the result is as good at BS 2099 as at BS 2000.

Four entry kinds:

- `computed` — a tithi rule in `rules.json`. Covers the festivals that matter most.
- `fixed` — the same BS month/day every year (Nepali New Year, the twelve sankrantis).
  A BS month begins on its sankranti by construction, so those need no astronomy.
- `ad_fixed` — the same Gregorian month/day every year (Labour Day, Christmas).
- `lunar` — a hand-verified date pinned to one BS year, which *overrides* the computed
  rule of the same name. Nepal's holiday gazette occasionally moves an observance for
  reasons no ephemeris predicts; this is where that goes, without giving up computation
  for the other ninety-nine years. `override_years` reports the range in use.

`festival_source` in the JSON reports `"computed"` and `festival_range` the years it
spans, so a consumer knows whether a festival-free month means "none" or "unknown".

Accuracy is pinned by `known_festival_dates_match` in `src/festivals.rs`, a table of
published Nepali festival dates the rules must reproduce. It deliberately includes the
awkward years: BS 2081 lost a tithi, so Maha Ashtami and Maha Navami share one day and
Kukur Tihar shares another with Laxmi Puja.

## Output behavior

- Calendar layout is `cal`-style with weekly rows.
- Today is resolved from the **local** date, not UTC. Nepal is UTC+05:45, so reading the
  clock in UTC reported yesterday's BS date between local midnight and 05:45.
- Header uses BS month name + year (for example, `Asar 2083`).
- Weekday header is `Su Mo Tu We Th Fr Sa`.
- Current BS day is highlighted using ANSI reverse video (`\x1b[7m...\x1b[0m`) when visible in the rendered month.

## Constraints

Date conversion and validation are constrained by the built-in BS reference table:

- Supported BS year range: `2000` to `2099` (inclusive)
- Supported month range: `1` to `12`
- Day must be valid for the selected BS month and year

Anchor mapping used by conversion:
- `BS 2000-01-01` = `AD 1943-04-14`

BS month lengths are fixed year by year by Nepal's Calendar Determination Committee and
cannot be derived, so they come from a table. It is cross-checked against two independent
published sources which agree on every year in range except BS 2089, where the wider of
the two is used. Rows are never extrapolated: when a year is not in a source, the table
stops rather than guessing.

Because the reference table is finite, conversions outside the covered date span are not supported.

## Development

Run tests:

```bash
cargo test
```
