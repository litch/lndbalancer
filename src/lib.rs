use anyhow::{anyhow, Result, Error};

use tonic_lnd::lnrpc::{ Channel };
use std::sync::{Arc, RwLock};

#[derive(Default, Debug, Copy, Clone)]
pub struct Config {
    pub dynamic_fees: bool,
    pub dynamic_fee_min: i64,
    pub dynamic_fee_max: i64,
    pub dynamic_fee_intervals: i8,
    pub dynamic_fee_update_frequency: i64,
}

impl Config {
    pub fn default() -> Config {
        Config {
            dynamic_fees: false,
            dynamic_fee_min: 0,
            dynamic_fee_max: 1000,
            dynamic_fee_intervals: 5,
            dynamic_fee_update_frequency: 14400,
        }
    }

    pub fn current() -> Arc<Config> {
        CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
    }
    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Config>> = RwLock::new(Default::default());
}


pub async fn calculate_htlc_max(channel: Channel, config: &Config) -> Result<u64, Error> {
    let ours: u64 = channel.local_balance as u64;
    let values = [
        1_000,
        100_000,
        250_000,
        1_000_000,
        10_000_000,
        50_000_000,
        100_000_000,
        250_000_000,
        500_000_000,
        1_000_000_000,
        2_000_000_000,
        3_000_000_000,
        4_000_000_000,
        5_000_000_000,
        7_500_000_000,
        10_000_000_000,
        15_000_000_000,
        20_000_000_000,
    ];
    let target = values.iter().rev().find(|&x| &ours >= x);
    let t = match target {
        Some(t) => t,
        None => &ours,
    };
    let mut capped = (0.9 * (*t as f64)).round();
    // set the minimum to 1_000
    if capped < 1_000.0 {
        capped = 1_000.0;
    }
    Ok(capped as u64)
}

pub async fn calculate_fee_target(channel: &Channel, config: &Config) -> Result<f64, Error> {
    let ours = channel.local_balance as f64;
    let total = channel.capacity as f64;
    let proportion = 1.0 - (ours / total);

    let max = config.dynamic_fee_max as f64;
    let min = config.dynamic_fee_min as f64;

    let range = max - min;

    let interval_size = (1.0 / config.dynamic_fee_intervals as f32) as f64;


    println!("Target Calculation (Ours: {}, Total: {}, Proportion: {}, Range: {})", ours, total, proportion, range);

    let parts = (proportion / interval_size).round() as f64;
    println!("Num Parts: {}, Proportion: {}, Interval Size: {}", parts, proportion, interval_size);

    let fee = min + ((range / config.dynamic_fee_intervals as f64) * parts);

    Ok(fee / 1000000.0)

    // Ok(8 as f64)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn config_default() {
        let config = Config::default();
        assert_eq!(config.dynamic_fee_max, 1000);
    }

    #[tokio::test]
    async fn calculate_htlc_max_channels() {
        let config = Config {
            dynamic_fees: true,
            dynamic_fee_update_frequency: 100,
            dynamic_fee_intervals: 5,
            dynamic_fee_min: 100,
            dynamic_fee_max: 500,
        };

        let test_cases = vec![
            (1_557_248_000, 900_000_000),
            (7_305_243_000, 4_500_000_000),
            (4_492_794_000, 3_600_000_000),
            (12_630_110_000, 9_000_000_000),
        ];


        for (ours, target) in test_cases {
            let c = Channel {
                local_balance: ours,
                remote_balance: target,
                ..Default::default()
            };
            let calc = calculate_htlc_max(c, &config).await.unwrap();
            assert_eq!(calc, target as u64);
        }
    }


    #[tokio::test]
    async fn calculate_fee_target_for_channels() {
        let config = Config {
            dynamic_fees: true,
            dynamic_fee_update_frequency: 100,
            dynamic_fee_intervals: 5,
            dynamic_fee_min: 10,
            dynamic_fee_max: 500,
        };

        let test_cases = vec![
            (1000, 1000, 1e-5),
            (1000, 0, 0.0005),
            (1000, 200, 0.000402),
            (1000, 205, 0.000402),
            (1000, 599, 0.000206),
            (1000, 605, 0.000206),
            (1000, 795, 0.000108),
        ];

        for (channel_size, ours, fee) in test_cases {
            let c = Channel {
                local_balance: ours,
                capacity: channel_size,
                ..Default::default()

            };

            let target = calculate_fee_target(&c, &config).await.unwrap();
            assert_eq!(target, fee)
        }
    }



}
