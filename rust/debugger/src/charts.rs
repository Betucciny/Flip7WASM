// ─────────────────────────────────────────────────────────────────────────────
// ASCII histogram
// ─────────────────────────────────────────────────────────────────────────────

const ASCII_BINS: usize = 22;
const BAR_MAX: usize = 44;

/// Prints a horizontal bar chart of `draw_scores` to stdout.
/// The bin that contains `stop_score` is annotated with `◄ STOP=N`.
pub fn print_ascii_histogram(draw_scores: &[i32], stop_score: i32) {
    if draw_scores.is_empty() {
        println!("  (no data)");
        return;
    }

    let x_max = (*draw_scores.iter().max().unwrap()).max(stop_score).max(1);
    let bin_w = (((x_max as f64) / (ASCII_BINS as f64)).ceil() as i32).max(1);
    let n_bins = ((x_max + bin_w - 1) / bin_w) as usize;

    let mut bins = vec![0u32; n_bins];
    for &s in draw_scores {
        let i = ((s as f64 / bin_w as f64).floor() as usize).min(n_bins - 1);
        bins[i] += 1;
    }

    let max_count = *bins.iter().max().unwrap_or(&1) as f64;
    let total = draw_scores.len() as f64;

    println!("  {}", "─".repeat(70));
    for (idx, &count) in bins.iter().enumerate() {
        let lo = idx as i32 * bin_w;
        let hi = lo + bin_w - 1;
        let bar_len = (count as f64 / max_count * BAR_MAX as f64).round() as usize;
        let in_stop = stop_score >= lo && stop_score <= hi;
        let pct = count as f64 / total * 100.0;

        // Only render bins that have data or mark the stop score.
        if count == 0 && !in_stop {
            continue;
        }

        let bar = format!("{:<width$}", "█".repeat(bar_len), width = BAR_MAX);
        let stop_tag = if in_stop {
            format!(" ◄ STOP={}", stop_score)
        } else {
            String::new()
        };

        println!(
            "  {:>4}–{:<4} │{}│ {:>5}  {:>5.1}%{}",
            lo, hi, bar, count, pct, stop_tag
        );
    }
    println!("  {}", "─".repeat(70));
}

// ─────────────────────────────────────────────────────────────────────────────
// SVG chart
// ─────────────────────────────────────────────────────────────────────────────

/// Saves a dark-themed SVG histogram to `path`.
///
/// Blue bars show the draw score distribution; a red dashed vertical line
/// marks the stop score so both options are visible on the same axis.
pub fn save_svg_chart(
    path: &str,
    title: &str,
    draw_scores: &[i32],
    stop_score: i32,
) -> Result<(), std::io::Error> {
    if draw_scores.is_empty() {
        return Ok(());
    }

    // ── Layout constants ──────────────────────────────────────────────────────
    let svg_w: f64 = 920.0;
    let svg_h: f64 = 530.0;
    let cl: f64 = 78.0; // chart left
    let cr: f64 = 880.0; // chart right
    let ct: f64 = 72.0; // chart top
    let cb: f64 = 435.0; // chart bottom
    let cw = cr - cl;
    let ch = cb - ct;
    const N_BINS: usize = 30;

    // ── Binning ───────────────────────────────────────────────────────────────
    let x_max = (*draw_scores.iter().max().unwrap())
        .max(stop_score + 5)
        .max(10) as f64;
    let n = draw_scores.len();
    let bw_score = x_max / N_BINS as f64;

    let mut bins = vec![0u32; N_BINS];
    for &s in draw_scores {
        let i = ((s as f64 / bw_score).floor() as usize).min(N_BINS - 1);
        bins[i] += 1;
    }
    let max_count = *bins.iter().max().unwrap_or(&1) as f64;

    // ── Coordinate helpers ────────────────────────────────────────────────────
    let xs = |score: f64| cl + (score / x_max) * cw;
    let bar_px_w = cw / N_BINS as f64;

    let mut v: Vec<String> = Vec::new();

    // ── SVG header + background ───────────────────────────────────────────────
    v.push(format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"##,
        svg_w, svg_h
    ));
    v.push(format!(
        r##"<rect width="{}" height="{}" fill="#1e1e2e"/>"##,
        svg_w, svg_h
    ));
    // Chart area background
    v.push(format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#181825" rx="4"/>"##,
        cl, ct, cw, ch
    ));

    // ── Title ─────────────────────────────────────────────────────────────────
    v.push(format!(
        r##"<text x="{:.0}" y="48" text-anchor="middle" font-family="monospace,sans-serif" font-size="17" font-weight="bold" fill="#cdd6f4">{}</text>"##,
        (cl + cr) / 2.0,
        xml_esc(title)
    ));

    // ── Horizontal grid lines + y-axis labels ─────────────────────────────────
    for i in 1..=5u32 {
        let frac = i as f64 / 5.0;
        let y = cb - frac * ch;
        let label = (max_count * frac).round() as u32;
        v.push(format!(
            r##"<line x1="{}" y1="{:.1}" x2="{}" y2="{:.1}" stroke="#45475a" stroke-width="1" stroke-dasharray="4,4"/>"##,
            cl, y, cr, y
        ));
        v.push(format!(
            r##"<text x="{:.1}" y="{:.1}" text-anchor="end" font-family="monospace" font-size="11" fill="#6c7086">{}</text>"##,
            cl - 5.0,
            y + 4.0,
            label
        ));
    }
    // Zero label
    v.push(format!(
        r##"<text x="{:.1}" y="{:.1}" text-anchor="end" font-family="monospace" font-size="11" fill="#6c7086">0</text>"##,
        cl - 5.0,
        cb + 4.0
    ));

    // ── Bars ──────────────────────────────────────────────────────────────────
    for (i, &count) in bins.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let bx = cl + i as f64 * bar_px_w;
        let bar_h = (count as f64 / max_count) * ch;
        let by = cb - bar_h;
        v.push(format!(
            r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#89b4fa" opacity="0.82" rx="2"/>"##,
            bx + 1.0,
            by,
            bar_px_w - 2.0,
            bar_h
        ));
    }

    // ── Stop score vertical line ──────────────────────────────────────────────
    let stop_px = xs(stop_score as f64);
    v.push(format!(
        r##"<line x1="{:.1}" y1="{}" x2="{:.1}" y2="{}" stroke="#f38ba8" stroke-width="2.5" stroke-dasharray="7,4"/>"##,
        stop_px, ct, stop_px, cb
    ));
    v.push(format!(
        r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-family="monospace" font-size="12" font-weight="bold" fill="#f38ba8">▲ {}</text>"##,
        stop_px,
        ct - 9.0,
        stop_score
    ));

    // ── X-axis ticks + labels ─────────────────────────────────────────────────
    let n_ticks = 10;
    for i in 0..=n_ticks {
        let score = x_max * i as f64 / n_ticks as f64;
        let px = xs(score);
        v.push(format!(
            r##"<line x1="{:.1}" y1="{}" x2="{:.1}" y2="{:.1}" stroke="#6c7086" stroke-width="1"/>"##,
            px,
            cb,
            px,
            cb + 5.0
        ));
        v.push(format!(
            r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-family="monospace" font-size="11" fill="#6c7086">{}</text>"##,
            px,
            cb + 17.0,
            score.round() as i32
        ));
    }

    // ── Axes ──────────────────────────────────────────────────────────────────
    v.push(format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#cdd6f4" stroke-width="1.5"/>"##,
        cl, cb, cr, cb
    ));
    v.push(format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#cdd6f4" stroke-width="1.5"/>"##,
        cl, ct, cl, cb
    ));

    // ── Axis labels ───────────────────────────────────────────────────────────
    v.push(format!(
        r##"<text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="monospace" font-size="13" fill="#a6adc8">Score</text>"##,
        (cl + cr) / 2.0,
        svg_h - 10.0
    ));
    let mid_y = (ct + cb) / 2.0;
    v.push(format!(
        r##"<text x="22" y="{:.0}" text-anchor="middle" font-family="monospace" font-size="13" fill="#a6adc8" transform="rotate(-90 22 {:.0})">Count</text>"##,
        mid_y, mid_y
    ));

    // ── Legend ────────────────────────────────────────────────────────────────
    let lx = cl + 10.0;
    let ly = cb + 48.0;
    v.push(format!(
        r##"<rect x="{:.0}" y="{:.0}" width="12" height="12" fill="#89b4fa" opacity="0.82" rx="2"/>"##,
        lx,
        ly - 10.0
    ));
    v.push(format!(
        r##"<text x="{:.0}" y="{:.0}" font-family="monospace" font-size="12" fill="#cdd6f4">Draw distribution (n={})</text>"##,
        lx + 18.0,
        ly,
        n
    ));
    v.push(format!(
        r##"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#f38ba8" stroke-width="2.5" stroke-dasharray="7,4"/>"##,
        lx + 220.0,
        ly - 5.0,
        lx + 250.0,
        ly - 5.0
    ));
    v.push(format!(
        r##"<text x="{:.0}" y="{:.0}" font-family="monospace" font-size="12" fill="#cdd6f4">Stop score ({})</text>"##,
        lx + 260.0,
        ly,
        stop_score
    ));

    v.push("</svg>".to_string());
    std::fs::write(path, v.join("\n"))
}

fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
