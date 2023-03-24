use std::fmt::Display;
use std::io::Write;
use std::{fs, str::FromStr, thread::sleep, time::Duration};

use anyhow::anyhow;
use env_logger::Env;
use log::{debug, info};

const LOW_THRESHOLD: i8 = 70;
const HIGH_THRESHOLD: i8 = 80;

fn main() -> Result<(), anyhow::Error> {
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            })
            .init(),
        _ => env_logger::Builder::from_env(Env::default().default_filter_or("info")).init(),
    };

    info!(
        "Starting up. Current charge behaviour is {}",
        get_behaviour()?
    );
    loop {
        let cap = get_capacity()?;
        let be = get_behaviour()?;
        let be_new = calc_behaviour(cap, &be);

        debug!("Battery capacity {cap}, behaviour {be}");
        if be != be_new {
            info!("Setting new charge behaviour: {be_new}. Old was {be}. battery at {cap}% . ");
            set_behaviour(be_new)?;
        }

        sleep(Duration::from_secs(60));
    }
}

fn get_capacity() -> Result<i8, anyhow::Error> {
    let s = fs::read_to_string("/sys/class/power_supply/macsmc-battery/capacity")?;
    let cap = s.trim().parse::<i8>()?;
    Ok(cap)
}

fn calc_behaviour(cap: i8, cb: &ChargeBehaviour) -> ChargeBehaviour {
    match (cap, cb) {
        // This should ensure that if we're > max we discharge until max and then inhibit,
        // and if we're < low then we'll charge all the way to max.
        (c, _) if c > HIGH_THRESHOLD => ChargeBehaviour::ForceDischarge,
        (c, _) if c < LOW_THRESHOLD => ChargeBehaviour::Auto,
        (c, ChargeBehaviour::Auto) if c < HIGH_THRESHOLD => ChargeBehaviour::Auto,
        (c, ChargeBehaviour::ForceDischarge) if c < HIGH_THRESHOLD => {
            ChargeBehaviour::InhibitCharge
        }
        (_, _) => ChargeBehaviour::InhibitCharge,
    }
}

fn get_behaviour() -> Result<ChargeBehaviour, anyhow::Error> {
    let s = fs::read_to_string("/sys/class/power_supply/macsmc-battery/charge_behaviour")?;
    let b = s.as_str().parse::<ChargeBehaviour>()?;
    Ok(b)
}

fn set_behaviour(b: ChargeBehaviour) -> Result<(), anyhow::Error> {
    fs::write(
        "/sys/class/power_supply/macsmc-battery/charge_behaviour",
        b.to_string(),
    )?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
enum ChargeBehaviour {
    Auto,
    ForceDischarge,
    InhibitCharge,
}

impl FromStr for ChargeBehaviour {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        match s {
            "auto" => Ok(Self::Auto),
            "force-discharge" => Ok(Self::ForceDischarge),
            "inhibit-charge" => Ok(Self::InhibitCharge),
            _ => Err(anyhow!("Unknown charge_behaviour!")),
        }
    }
}

impl Display for ChargeBehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ChargeBehaviour::Auto => "auto",
            ChargeBehaviour::ForceDischarge => "force-discharge",
            ChargeBehaviour::InhibitCharge => "inhibit-charge",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::{calc_behaviour, ChargeBehaviour, HIGH_THRESHOLD, LOW_THRESHOLD};

    #[test]
    fn calculate_from_force_discharge_behaviour() {
        assert_eq!(
            ChargeBehaviour::ForceDischarge,
            calc_behaviour(HIGH_THRESHOLD + 1, &ChargeBehaviour::ForceDischarge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(HIGH_THRESHOLD, &ChargeBehaviour::ForceDischarge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(HIGH_THRESHOLD - 1, &ChargeBehaviour::ForceDischarge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(LOW_THRESHOLD + 1, &ChargeBehaviour::ForceDischarge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(LOW_THRESHOLD, &ChargeBehaviour::ForceDischarge)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(LOW_THRESHOLD - 1, &ChargeBehaviour::ForceDischarge)
        );
    }

    #[test]
    fn calculate_from_inhibit_behaviour() {
        assert_eq!(
            ChargeBehaviour::ForceDischarge,
            calc_behaviour(HIGH_THRESHOLD + 1, &ChargeBehaviour::InhibitCharge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(HIGH_THRESHOLD, &ChargeBehaviour::InhibitCharge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(HIGH_THRESHOLD - 1, &ChargeBehaviour::InhibitCharge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(LOW_THRESHOLD + 1, &ChargeBehaviour::InhibitCharge)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(LOW_THRESHOLD, &ChargeBehaviour::InhibitCharge)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(LOW_THRESHOLD - 1, &ChargeBehaviour::InhibitCharge)
        );
    }

    #[test]
    fn calculate_from_auto_behaviour() {
        assert_eq!(
            ChargeBehaviour::ForceDischarge,
            calc_behaviour(HIGH_THRESHOLD + 1, &ChargeBehaviour::Auto)
        );
        assert_eq!(
            ChargeBehaviour::InhibitCharge,
            calc_behaviour(HIGH_THRESHOLD, &ChargeBehaviour::Auto)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(HIGH_THRESHOLD - 1, &ChargeBehaviour::Auto)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(LOW_THRESHOLD + 1, &ChargeBehaviour::Auto)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(LOW_THRESHOLD, &ChargeBehaviour::Auto)
        );
        assert_eq!(
            ChargeBehaviour::Auto,
            calc_behaviour(LOW_THRESHOLD - 1, &ChargeBehaviour::Auto)
        );
    }

    #[test]
    fn verify_formatting_of_enum() {
        assert_eq!("auto", ChargeBehaviour::Auto.to_string());
        assert_eq!(
            "force-discharge",
            ChargeBehaviour::ForceDischarge.to_string()
        );
        assert_eq!("inhibit-charge", ChargeBehaviour::InhibitCharge.to_string());

        let s = "force-discharge";
        let p = s.parse::<ChargeBehaviour>().unwrap();
        assert_eq!(s, p.to_string());
    }
}
