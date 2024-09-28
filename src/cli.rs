use clap::Parser;
use clap::ValueEnum;
use std::str::FromStr;
use strum::Display;

#[derive(ValueEnum, Debug, Clone, Display)]
// #[clap(rename_all = "kebab_case")]
pub enum Unit {
    Ms,
    S,
    MB,
    KB,
    GB,
    KiB,
    MiB,
    GiB,
}

impl FromStr for Unit {
    type Err = ();

    fn from_str(input: &str) -> Result<Unit, Self::Err> {
        match input.to_lowercase().as_str() {
            "ms" => Ok(Unit::Ms),
            "s" => Ok(Unit::S),
            "mb" => Ok(Unit::MB),
            "kb" => Ok(Unit::KB),
            "gb" => Ok(Unit::GB),
            "kib" => Ok(Unit::KiB),
            "mib" => Ok(Unit::MiB),
            "gib" => Ok(Unit::GiB),
            _ => Err(()),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Chart title, will be shown at the top of the chart
    #[arg(short, long, value_name = "STRING")]
    pub title: Option<String>,

    /// Unit to be used in the chart
    #[arg(short, long)]
    pub unit: Vec<Unit>,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}"
    )
}
