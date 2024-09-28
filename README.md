# ttydash

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/550W-HOST/ttydash/ci.yml?style=flat-square&logo=github)
 ![Crates.io Version](https://img.shields.io/crates/v/ttydash?style=flat-square&logo=rust)
 ![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/ttydash?style=flat-square)
[![dependency status](https://deps.rs/repo/github/550w-host/ttydash/status.svg?style=flat-square)](https://deps.rs/repo/github/550w-host/ttydash)
![Crates.io License](https://img.shields.io/crates/l/ttydash?style=flat-square) ![Crates.io Size](https://img.shields.io/crates/size/ttydash?style=flat-square)



## Snapshot

![ttydash](./assets/Snipaste.png)

## Usage

### Example 1
```bash
ping 8.8.8.8 | ttydash -u ms
```

## flags

```bash
A rust based tty plot

Usage: ttydash [OPTIONS]

Options:
      --tick-rate <FLOAT>   Tick rate, i.e. number of ticks per second [default: 4]
  -f, --frame-rate <FLOAT>  Frame rate, i.e. number of frames per second [default: 60]
  -t, --title <STRING>      Chart title, will be shown at the top of the chart
  -u, --unit <UNIT>         Unit to be used in the chart [possible values: ms, s, mb, kb, gb, ki-b, mi-b, gi-b]
  -h, --help                Print help
  -V, --version             Print version
```