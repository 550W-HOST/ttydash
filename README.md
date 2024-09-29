# ttydash

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/550W-HOST/ttydash/ci.yml?style=flat-square&logo=github)
 ![Crates.io Version](https://img.shields.io/crates/v/ttydash?style=flat-square&logo=rust)
 ![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/ttydash?style=flat-square)
[![dependency status](https://deps.rs/repo/github/550w-host/ttydash/status.svg?style=flat-square)](https://deps.rs/repo/github/550w-host/ttydash)
![Crates.io License](https://img.shields.io/crates/l/ttydash?style=flat-square) ![Crates.io Size](https://img.shields.io/crates/size/ttydash?style=flat-square)

## Snapshot

![ttydash](./assets/Snipaste.png)

## Installation

```bash
cargo install ttydash
```

## **`ttydash` Usage Guide**

### **Single Line Data Input**

#### **Pure Data Input** (No Title, No Unit)
To input pure data continuously, use the following command:
```bash
while true; echo 1; sleep 0.5; end | ttydash
```

#### **Adding a Title** (Optional)
If you want to add a title to the data chart, just use the `-t` flag:
```bash
while true; echo 1; sleep 0.5; end | ttydash -t "🌟 Title"
```

#### **Adding Units** (Optional)
If each line of data comes with a unit (e.g., "ms"), you can specify the unit with the `-u` flag. The unit can be one of the following:

- ⏱️ `ms` (milliseconds)
- ⏳ `s` (seconds)
- 📦 `mb` (megabytes)
- 🧑‍💻 `kb` (kilobytes)
- 💽 `gb` (gigabytes)
- 📊 `ki-b` (kibibytes)

Example 1️:
```bash
while true; echo 1ms; sleep 0.5; end | ttydash -u ms
```
Example 2️:
```bash
while true; echo 1 ms; sleep 0.5; end | ttydash -u ms
```
👉 Note: The space between the number and the unit is optional.

### ➕ **Multiple Data Points** on the Same Line
To input multiple data points at once, just separate them with a space. For example:
```bash
while true; echo "1 2 3"; sleep 0.5; end | ttydash
```

📊 `ttydash` will plot the data points in the order they are provided!

### 🎯 **Plot Specific Data Points** Using the `-i` Flag
If you only want to plot specific data points, you can use the `-i` flag to select their index. For example:
```bash
while true; echo "1 2 3"; sleep 0.5; end | ttydash -i 1 2
```
In this example, only the data at **index 1** and **index 2** will be plotted.

👉 **Note**: You can switch the sequence of the index as needed. For example:
```bash
ttydash -i 2 1
```
This will plot **index 2** first, followed by **index 1**.


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