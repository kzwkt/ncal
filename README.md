# ncal

`ncal` is a terminal Bikram Sambat (BS) calendar tool, inspired by Unix `cal`.



It supports:
- AD <-> BS date conversion internally
- current-day highlighting in calendar output

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
ncal [day] [month] [year]
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

## Output behavior

- Calendar layout is `cal`-style with weekly rows.
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
