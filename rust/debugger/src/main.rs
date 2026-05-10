mod analysis;
mod charts;
mod scenarios;

use engine::{
    game::{actions::Action, scoring::calculate_score},
    simulation::{bust_probability_for_player, score_distribution},
};

const N_SIMS: u32 = 2_000;

fn main() {
    std::fs::create_dir_all("charts").unwrap_or(());

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║          Flip7  ·  Score Distribution Debugger                  ║");
    println!(
        "║          {} simulations per scenario                          ║",
        N_SIMS
    );
    println!("╚══════════════════════════════════════════════════════════════════╝");

    for scenario in scenarios::all_scenarios() {
        run_scenario(&scenario);
    }

    println!("\n  SVG charts saved to ./charts/  (open in any browser)\n");
}

fn run_scenario(scenario: &scenarios::Scenario) {
    let state = &scenario.state;
    let pid = scenario.player_id;
    let player = &state.players[pid];

    let stop_score = calculate_score(player);
    let bust_prob = bust_probability_for_player(state, pid);
    let draw_scores = score_distribution(state, pid, Action::Draw, N_SIMS);

    // ── Header ────────────────────────────────────────────────────────────────
    println!("\n  ┌──────────────────────────────────────────────────────────────┐");
    println!("  │  {:<62}│", scenario.name);
    println!("  └──────────────────────────────────────────────────────────────┘");
    println!("  {}", scenario.description);
    println!();

    // ── Hand summary ──────────────────────────────────────────────────────────
    println!("  Numbers   : {:?}", player.numbers);
    println!("  Modifiers : {:?}", player.modifiers);
    println!("  Lifeline  : {}", player.has_lifeline);
    println!(
        "  Draw pile : {} cards remaining",
        state.deck.draw_pile.len()
    );
    println!(
        "  Stop score: {}   Bust prob: {:.1}%",
        stop_score,
        bust_prob * 100.0
    );
    println!();

    if draw_scores.is_empty() {
        println!("  (no valid draw outcomes — check state validity)");
        return;
    }

    // ── Stats table ───────────────────────────────────────────────────────────
    let stats = analysis::compute_stats(&draw_scores).unwrap();
    print_stats(&stats, stop_score);
    println!();

    // ── ASCII histogram ───────────────────────────────────────────────────────
    charts::print_ascii_histogram(&draw_scores, stop_score);

    // ── SVG chart ─────────────────────────────────────────────────────────────
    let slug: String = scenario
        .name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let slug = slug.trim_matches('_').to_string();
    let path = format!("charts/{}.svg", slug);

    match charts::save_svg_chart(&path, &scenario.name, &draw_scores, stop_score) {
        Ok(()) => println!("  ✓ {}", path),
        Err(e) => println!("  ✗ chart error: {}", e),
    }
}

fn print_stats(stats: &analysis::Stats, stop_score: i32) {
    let gain = stats.mean - stop_score as f64;
    let sign = if gain >= 0.0 { "+" } else { "" };

    println!(
        "  ── Draw distribution (n={}) ─────────────────────────────────",
        stats.count
    );
    println!(
        "  Mean {:>7.1}   Median {:>7.1}   Std {:>6.1}",
        stats.mean, stats.median, stats.std_dev
    );
    println!(
        "  P10  {:>7.1}   P25    {:>7.1}   P75 {:>6.1}   P90 {:>6.1}",
        stats.p10, stats.p25, stats.p75, stats.p90
    );
    println!(
        "  Min  {:>7}   Max    {:>7}   Bust rate {:>5.1}%",
        stats.min,
        stats.max,
        stats.bust_rate * 100.0
    );
    println!(
        "  Expected gain vs stop  ({:.1} − {} = {}{:.1})",
        stats.mean, stop_score, sign, gain
    );
    println!("  {}", "─".repeat(60));
}
