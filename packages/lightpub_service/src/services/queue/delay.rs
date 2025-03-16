use once_cell::sync::Lazy;
use std::time::Duration;

pub const APUB_DELIVERY_MAX_ATTEMPTS: u64 = 10;
const APUB_DELIVERY_INIT_DELAY: f64 = 5.0;

pub static APUB_DELIVERY_DELAY_TIMES: Lazy<Vec<Duration>> = Lazy::new(|| {
    calculate_exponential_backoff_delays(APUB_DELIVERY_INIT_DELAY, 3.5, APUB_DELIVERY_MAX_ATTEMPTS)
});

pub fn calculate_exponential_backoff_delays(
    initial_delay_seconds: f64,
    multiplier: f64,
    max_attempts: u64,
) -> Vec<Duration> {
    let mut delays = Vec::with_capacity(max_attempts as usize);

    for attempt in 0..max_attempts {
        // Calculate the delay for this attempt using the exponential formula
        let delay_seconds = initial_delay_seconds * multiplier.powi(attempt as i32);

        // Convert to u64 (rounded to nearest second)
        let delay_seconds_u64 = delay_seconds.round() as u64;

        delays.push(Duration::from_secs(delay_seconds_u64));
    }

    assert!(delays.len() == max_attempts as usize);

    delays
}
