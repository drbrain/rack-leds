use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Prometheus source
    #[arg(short, long, value_name = "URL")]
    pub source: String,

    /// Prometheus refresh period
    #[arg(long, value_name = "SECONDS", value_parser = secs)]
    period: Option<Duration>,

    /// Prometheus query timeout in milliseconds
    #[arg(long, value_name = "MILLISECONDS", value_parser = millis)]
    timeout: Option<Duration>,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Tick rate, i.e. number of ticks per second
    #[arg(long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,
}

impl Args {
    pub fn period(&self) -> Duration {
        self.period.unwrap_or_else(|| Duration::from_secs(15))
    }

    pub fn timeout(&self) -> Duration {
        self.timeout.unwrap_or_else(|| Duration::from_millis(100))
    }
}

fn millis(millis: &str) -> Result<Duration, String> {
    let millis: u64 = millis
        .parse()
        .map_err(|_| format!("{millis} isn't a valid millisecond duration"))?;

    Ok(Duration::from_millis(millis))
}

fn secs(secs: &str) -> Result<Duration, String> {
    let secs: u64 = secs
        .parse()
        .map_err(|_| format!("{secs} isn't a valid seconds duration"))?;

    Ok(Duration::from_secs(secs))
}
