# ncal

`ncal` is a terminal Bikram Sambat (BS) calendar tool, inspired by Unix `cal`.



It supports:
- AD <-> BS date conversion, in both directions
- current-day highlighting in calendar output
- Devanagari month names, weekday names and numerals
- festivals and public holidays
- JSON output, so other programs can build on it

## Build and run

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
  "ncal": "0.2.0",
  "range": { "bs_start": 2000, "bs_end": 2090, "lunar_years_covered": null },
  "today": {
    "bs": { "year": 2083, "month": 5, "day": 13 },
    "ad": "2026-08-29", "weekday": 6,
    "month_name": "Bhadra", "month_name_np": "भदौ",
    "weekday_name": "Saturday", "weekday_name_np": "शनिबार",
    "bs_day_np": "१३", "bs_year_np": "२०८३",
    "is_holiday": true, "festivals": []
  },
  "month": {
    "year": 2083, "month": 5, "days": 31, "name": "Bhadra", "name_np": "भदौ",
    "start_ad": "2026-08-17", "end_ad": "2026-09-16",
    "weeks": [ [ null, null, { "bs_day": 1, "bs_day_np": "१", "ad": "2026-08-17",
                               "ad_day": 17, "ad_month_short": "Aug", "weekday": 2,
                               "is_today": false, "is_holiday": false,
                               "festivals": [] } ] ]
  },
  "festivals": [ { "bs_day": 1, "en": "Nepali New Year", "np": "नयाँ वर्ष", "holiday": true } ]
}
```

Each week is exactly seven entries, Sunday first, padded with `null` at both ends.
Weekdays count from Sunday = 0. Treat these field names as a contract.

## Festivals

Festival data lives in `src/data/festivals.json` and is embedded into the binary, so
there is no runtime file dependency and no network access. Three entry kinds:

- `fixed` — the same BS month/day every year (Nepali New Year, Constitution Day).
  Rule-based, so correct for every supported year.
- `ad_fixed` — the same Gregorian month/day every year (Labour Day, Christmas).
- `lunar` — tithi-based festivals, pinned to one BS year.

**Dashain, Tihar, Teej and the other tithi-based festivals are lunar.** They cannot be
derived from the reference table and have to be entered per year from an authoritative
panchanga. `lunar_years_covered` reports the inclusive BS year range for which that has
been done — it is `null` while no lunar entries exist, so a consumer can say "festival
data unavailable for this year" instead of showing an empty month as though the year
genuinely had none.

## Output behavior

- Calendar layout is `cal`-style with weekly rows.
- Today is resolved from the **local** date, not UTC. Nepal is UTC+05:45, so reading the
  clock in UTC reported yesterday's BS date between local midnight and 05:45.
- Header uses BS month name + year (for example, `Asar 2083`).
- Weekday header is `Su Mo Tu We Th Fr Sa`.
- Current BS day is highlighted using ANSI reverse video (`\x1b[7m...\x1b[0m`) when visible in the rendered month.

## Constraints

Date conversion and validation are constrained by the built-in BS reference table:

- Supported BS year range: `2000` to `2090` (inclusive)
- Supported month range: `1` to `12`
- Day must be valid for the selected BS month and year

Anchor mapping used by conversion:
- `BS 2000-01-01` = `AD 1943-04-14`

Because the reference table is finite, conversions outside the covered date span are not supported.

## Development

Run tests:

```bash
cargo test
```
