use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use strum::Display;

use crate::config::get_config_dir;
use crate::config::get_data_dir;

#[derive(Debug, Clone, Display)]
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

impl ValueEnum for Unit {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Unit::Ms,
            Unit::S,
            Unit::MB,
            Unit::KB,
            Unit::GB,
            Unit::KiB,
            Unit::MiB,
            Unit::GiB,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.to_string()))
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        match input {
            "ms" => Ok(Unit::Ms),
            "s" => Ok(Unit::S),
            "mb" => Ok(Unit::MB),
            "kb" => Ok(Unit::KB),
            "gb" => Ok(Unit::GB),
            "kib" => Ok(Unit::KiB),
            "mib" => Ok(Unit::MiB),
            "gib" => Ok(Unit::GiB),
            _ => Err(format!("Unknown unit: {}", input)),
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
    pub titles: Option<Vec<String>>,

    /// Unit to be used in the chart
    #[arg(short, long)]
    pub units: Option<Vec<Unit>>,

    /// Index vector to be used in the chart
    #[arg(short, long, value_name = "INT")]
    pub indices: Option<Vec<usize>>,

    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new regex to the list of regexes
    Add(AddArgs),
    /// Remove a regex from the list of regexes
    Remove(RemoveArgs),
    /// List all regexes
    List,
}
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Name of the regex
    #[arg(short, long)]
    pub name: String,
    /// The regex to add
    #[arg(short, long)]
    pub regex: String,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// The name of the regex to remove
    #[arg(short, long)]
    name: String,
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

    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
