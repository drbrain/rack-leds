use std::time::Duration;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Prometheus source
    #[arg(short, long)]
    pub source: String,

    /// Prometheus refresh period
    #[arg(long, value_parser = secs)]
    period: Option<Duration>,

    /// Prometheus query timeout in milliseconds
    #[arg(long, value_parser = millis)]
    timeout: Option<Duration>,
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
