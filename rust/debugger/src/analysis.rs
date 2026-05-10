/// Summary statistics for a collection of scores.
#[derive(Debug)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: i32,
    pub max: i32,
    pub p10: f64,
    pub p25: f64,
    pub p75: f64,
    pub p90: f64,
    /// Fraction of outcomes where the player scored 0 (busted or ended
    /// with no numbers).
    pub bust_rate: f64,
}

pub fn compute_stats(scores: &[i32]) -> Option<Stats> {
    if scores.is_empty() {
        return None;
    }

    let count = scores.len();
    let sum: i64 = scores.iter().map(|&s| s as i64).sum();
    let mean = sum as f64 / count as f64;

    let variance: f64 = scores
        .iter()
        .map(|&s| {
            let d = s as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / count as f64;
    let std_dev = variance.sqrt();

    let mut sorted = scores.to_vec();
    sorted.sort_unstable();

    let min = sorted[0];
    let max = *sorted.last().unwrap();
    let bust_count = scores.iter().filter(|&&s| s == 0).count();

    Some(Stats {
        count,
        mean,
        median: percentile(&sorted, 50.0),
        std_dev,
        min,
        max,
        p10: percentile(&sorted, 10.0),
        p25: percentile(&sorted, 25.0),
        p75: percentile(&sorted, 75.0),
        p90: percentile(&sorted, 90.0),
        bust_rate: bust_count as f64 / count as f64,
    })
}

fn percentile(sorted: &[i32], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    let rank = p / 100.0 * (n - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = (rank.ceil() as usize).min(n - 1);
    let frac = rank - lo as f64;
    sorted[lo] as f64 * (1.0 - frac) + sorted[hi] as f64 * frac
}
