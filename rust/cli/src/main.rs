use std::io::{self, Write};

use engine::{
    game::{
        actions::Action,
        cards::{ActionCard, Card, ModifierCard},
        engine::apply_action,
        scoring::calculate_score,
        state::{GamePhase, GameState, PendingEffect, PlayerStatus},
        turns::{end_round, new_game},
    },
    simulation::{RecommendedAction, bust_probability_for_current_player, recommend},
};

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    print_banner();
    loop {
        let n = read_usize("  Players (2–6): ", 2, 6);
        run_game(n);
        if !ask_yn("\n  New game?", false) {
            break;
        }
    }
    println!("\n  Goodbye!\n");
}

// ── Game / round loops ────────────────────────────────────────────────────────

fn run_game(player_count: usize) {
    let mut state = new_game(player_count);
    println!("\n  Game started with {} players.\n", player_count);
    loop {
        run_round(&mut state);
        print_round_results(&state);
        // Always commit round scores before any further checks.
        end_round(&mut state);

        // 200-point win condition — checked after scores are official.
        if let Some(winner) = find_winner(&state) {
            println!("\n  🏆  Player {winner} reached 200 points — GAME OVER!\n");
            break;
        }

        if !ask_yn(&format!("  Start round {}?", state.round), true) {
            break;
        }
    }
    print_final_scores(&state);
}

/// Returns the index of the player with the highest score ≥ 200, or `None` if
/// nobody has crossed the threshold yet.
fn find_winner(state: &GameState) -> Option<usize> {
    state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.total_score >= 200)
        .max_by_key(|(_, p)| p.total_score)
        .map(|(i, _)| i)
}

fn run_round(state: &mut GameState) {
    print_round_header(state);
    while state.phase == GamePhase::Playing {
        print_board(state);
        print_player_detail(state);
        // Compute and display recommendation once per "frame".
        let rec = recommend(state, 500);
        if let Some(ref r) = rec {
            print_advice(r);
        }
        // Keep prompting until a valid game action is accepted.
        loop {
            match handle_input(&read_line("  > "), state) {
                InputResult::Applied => break,
                InputResult::Refresh => {
                    // Re-run recommendation and re-prompt without redrawing board.
                    if let Some(ref r) = recommend(state, 500) {
                        print_advice(r);
                    }
                }
                InputResult::Err(msg) => println!("  ! {}\n", msg),
            }
        }
    }
}

// ── Command handler ───────────────────────────────────────────────────────────

enum InputResult {
    Applied,
    Refresh,
    Err(String),
}

fn handle_input(line: &str, state: &mut GameState) -> InputResult {
    let toks: Vec<&str> = line.trim().split_whitespace().collect();
    let cmd = match toks.first() {
        Some(c) => c.to_ascii_lowercase(),
        None => return InputResult::Err("No command. Type 'h' for help.".into()),
    };
    let pid = state.current_player;

    match cmd.as_str() {
        // ── Draw actions ─────────────────────────────────────────────────────
        "d" | "draw" => do_action(state, pid, Action::Draw),

        "k" | "known" => match parse_card(&toks[1..]) {
            Some(card) => do_action(state, pid, Action::DrawKnown { card }),
            None => InputResult::Err(
                "Usage: k n <0-12>  |  k a lif/frz/tp3  |  k m 2/4/6/8/10/x2".into(),
            ),
        },

        // ── Known tap3 (real-game tracking) ──────────────────────────────────
        "kt" | "tap3known" => match parse_tap3_known(&toks[1..]) {
            Some((target, cards)) => do_action(state, pid, Action::Tap3Known { target, cards }),
            None => InputResult::Err(
                "Usage: kt <target> <type> <val> [<type> <val>] [<type> <val>]".into(),
            ),
        },

        // ── Stop ─────────────────────────────────────────────────────────────
        "s" | "stop" => do_action(state, pid, Action::Stop),
        "f" | "freeze" => match parse_idx(&toks) {
            Some(t) => do_action(state, pid, Action::Freeze { target: t }),
            None => InputResult::Err("Usage: freeze <player_number>".into()),
        },
        "t" | "tap3" => match parse_idx(&toks) {
            Some(t) => do_action(state, pid, Action::Tap3 { target: t }),
            None => InputResult::Err("Usage: tap3 <player_number>".into()),
        },

        // ── Pending-effect resolution ─────────────────────────────────────────
        // ── Info / meta ───────────────────────────────────────────────────────
        "r" | "rec" => InputResult::Refresh,
        "h" | "help" | "?" => {
            print_help(state);
            InputResult::Refresh
        }
        "q" | "quit" | "exit" => {
            println!("\n  Exiting.\n");
            std::process::exit(0);
        }
        other => InputResult::Err(format!("Unknown command '{other}'. Type 'h' for help.")),
    }
}

fn do_action(state: &mut GameState, pid: usize, action: Action) -> InputResult {
    match apply_action(state.clone(), pid, action) {
        Ok(new) => {
            *state = new;
            InputResult::Applied
        }
        Err(e) => InputResult::Err(format!("{e:?}")),
    }
}

// ── Card / index parsing ──────────────────────────────────────────────────────

fn parse_card(toks: &[&str]) -> Option<Card> {
    match toks.first()?.to_ascii_lowercase().as_str() {
        "n" => {
            let n: u8 = toks.get(1)?.parse().ok()?;
            (n <= 12).then_some(Card::Number(n))
        }
        "a" => match toks.get(1)?.to_ascii_lowercase().as_str() {
            "l" | "lif" | "lifeline" => Some(Card::Action(ActionCard::Lifeline)),
            "f" | "frz" | "freeze" => Some(Card::Action(ActionCard::Freeze)),
            "t" | "tp3" | "tap3" => Some(Card::Action(ActionCard::Tap3)),
            _ => None,
        },
        "m" => match toks.get(1)?.to_ascii_lowercase().as_str() {
            "2" => Some(Card::Modifier(ModifierCard::Add(2))),
            "4" => Some(Card::Modifier(ModifierCard::Add(4))),
            "6" => Some(Card::Modifier(ModifierCard::Add(6))),
            "8" => Some(Card::Modifier(ModifierCard::Add(8))),
            "10" => Some(Card::Modifier(ModifierCard::Add(10))),
            "x2" | "*2" | "mul" | "multiply2" => Some(Card::Modifier(ModifierCard::Multiply2)),
            _ => None,
        },
        _ => None,
    }
}

fn parse_idx(toks: &[&str]) -> Option<usize> {
    toks.get(1)?.parse().ok()
}

/// Parses `kt` arguments: `<target> [<type> <val>]{1,3}`
///
/// Example: `kt 2 n 5 n 3 a lif`
fn parse_tap3_known(toks: &[&str]) -> Option<(usize, Vec<Card>)> {
    let target: usize = toks.first()?.parse().ok()?;
    let mut cards = Vec::new();
    let mut i = 1; // toks[0] was the target
    while i + 1 < toks.len() && cards.len() < 3 {
        match parse_card(&toks[i..i + 2]) {
            Some(card) => {
                cards.push(card);
                i += 2;
            }
            None => return None,
        }
    }
    if cards.is_empty() {
        None
    } else {
        Some((target, cards))
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

const W: usize = 60; // inner box width

fn rule(c: char) -> String {
    std::iter::repeat(c).take(W).collect::<String>()
}

fn print_banner() {
    println!();
    println!("  ╔{}╗", rule('═'));
    println!("  ║{:^W$}║", "FLIP 7  ·  Advisor CLI");
    println!("  ╠{}╣", rule('═'));
    println!(
        "  ║{:^W$}║",
        "draw = random sim   |   known = real-game tracking"
    );
    println!("  ╚{}╝", rule('═'));
    println!();
}

fn print_round_header(state: &GameState) {
    println!();
    println!("  ┌{}┐", rule('─'));
    println!(
        "  │{:^W$}│",
        format!(
            "Round {}   draw pile: {} cards   discard: {} cards",
            state.round,
            state.deck.draw_pile.len(),
            state.deck.discard_pile.len()
        )
    );
    println!("  └{}┘", rule('─'));
}

fn print_board(state: &GameState) {
    println!();
    for (i, p) in state.players.iter().enumerate() {
        let arrow = if i == state.current_player {
            "▶"
        } else {
            " "
        };
        let status = match p.status {
            PlayerStatus::Active => "Active  ",
            PlayerStatus::Stopped => "Stopped ",
            PlayerStatus::Busted => "Busted  ",
            PlayerStatus::Flip7 => "FLIP 7! ",
        };
        let nums: Vec<String> = p.numbers.iter().map(|n| format!("{n:2}")).collect();
        let nums_str = if nums.is_empty() {
            " —".to_string()
        } else {
            nums.join(" ")
        };
        let life = if p.has_lifeline { " 🛡" } else { "  " };
        let score = calculate_score(p);
        println!(
            "  {arrow} P{i}  {status}  tot:{:4}  score:{:3}{life}  [{}]",
            p.total_score, score, nums_str
        );
    }
    println!();
}

fn print_player_detail(state: &GameState) {
    let pid = state.current_player;
    let p = &state.players[pid];

    // Pending effect banner
    if let Some(effect) = &state.pending_effect {
        let name = match effect {
            PendingEffect::Freeze => "FREEZE",
            PendingEffect::Tap3 => "TAP 3",
        };
        println!("  ⚡  Pending: {name}  — choose a target player");
        for (i, op) in state.players.iter().enumerate() {
            if i != pid && op.status == PlayerStatus::Active {
                let nums: Vec<String> = op.numbers.iter().map(|n| n.to_string()).collect();
                println!(
                    "       P{i}  score: {:3}  [{}]",
                    calculate_score(op),
                    nums.join(" ")
                );
            }
        }
        println!();
        return;
    }

    // Normal turn detail
    let mods: Vec<String> = p
        .modifiers
        .iter()
        .map(|m| match m {
            ModifierCard::Add(v) => format!("+{v}"),
            ModifierCard::Multiply2 => "×2".to_string(),
        })
        .collect();
    let nums: Vec<String> = p.numbers.iter().map(|n| n.to_string()).collect();

    println!("  ── Player {pid}'s turn ─────────────────────────────────────");
    println!(
        "  Numbers:   {}",
        if nums.is_empty() {
            "(none)".into()
        } else {
            nums.join("  ")
        }
    );
    println!(
        "  Modifiers: {}",
        if mods.is_empty() {
            "(none)".into()
        } else {
            mods.join("  ")
        }
    );
    println!(
        "  Score: {}    Lifeline: {}    Bust risk: {:.1}%",
        calculate_score(p),
        if p.has_lifeline { "yes 🛡" } else { "no" },
        bust_probability_for_current_player(state) * 100.0,
    );
    println!();
}

fn print_advice(rec: &engine::simulation::Recommendation) {
    println!(
        "  ┌─ Advisor {}┐",
        rule('─').chars().take(W - 10).collect::<String>()
    );
    match &rec.action {
        RecommendedAction::Draw => {
            let d = rec.expected_score_draw.unwrap_or(0.0);
            let s = rec.expected_score_stop.unwrap_or(0.0);
            println!("  │  ✦ DRAW  (draw avg: {d:.1} pts  vs  stop avg: {s:.1} pts)");
        }
        RecommendedAction::Stop => {
            let d = rec.expected_score_draw.unwrap_or(0.0);
            let s = rec.expected_score_stop.unwrap_or(0.0);
            println!("  │  ✦ STOP  (stop avg: {s:.1} pts  vs  draw avg: {d:.1} pts)");
        }
        RecommendedAction::Target { index } => {
            println!("  │  ✦ Target Player {index}  (best expected outcome for you)");
        }
    }
    println!(
        "  │    Bust risk on next card: {:.1}%",
        rec.bust_probability * 100.0
    );
    println!("  └{}┘", rule('─'));
    println!();
}

fn print_help(state: &GameState) {
    println!();
    println!("  ── Commands ────────────────────────────────────────────────");
    println!("  d               random draw (simulation / unknown card)");
    println!("  k n <0-12>      draw a specific number card");
    println!("  k a lif         draw Lifeline action card");
    println!("  k a frz         draw Freeze action card");
    println!("  k a tp3         draw Tap3 action card");
    println!("  k m 2/4/6/8/10  draw an Add modifier card");
    println!("  k m x2          draw the ×2 modifier card");
    println!("  s               stop — lock in your current score");
    match state.pending_effect.as_ref() {
        Some(PendingEffect::Freeze) => println!("  f <n>           <- REQUIRED: freeze player n"),
        Some(PendingEffect::Tap3) => {
            println!("  t <n>           <- REQUIRED: tap3 player n  (random draw)");
            println!("  kt <n> <cards>  <- REQUIRED: tap3 with known cards");
            println!("                    e.g.  kt 2 n 5 n 3 a lif");
        }
        None => {
            println!("  f <n>           freeze player n  (after drawing Freeze)");
            println!("  t <n>           tap3 player n   (random  — after drawing Tap3)");
            println!("  kt <n> <cards>  tap3 with known cards (up to 3 card specs)");
        }
    }
    println!("  r               re-run advisor recommendation");
    println!("  h               show this help");
    println!("  q               quit");
    println!();
}

fn print_round_results(state: &GameState) {
    println!();
    println!(
        "  ══ Round {} Results ══════════════════════════════════",
        state.round
    );
    println!();

    let mut rows: Vec<(usize, i32, &PlayerStatus)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let round_pts = if p.status == PlayerStatus::Busted {
                0
            } else {
                calculate_score(p)
            };
            (i, round_pts, &p.status)
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1));

    for (i, round_pts, status) in &rows {
        let tag = match status {
            PlayerStatus::Busted => "BUST ",
            PlayerStatus::Flip7 => "FLIP7",
            _ => "  ok ",
        };
        let p = &state.players[*i];
        println!(
            "  P{i}  [{tag}]  this round: {:+4}  new total: {:4}",
            round_pts,
            p.total_score + round_pts
        );
    }
    println!();
}

fn print_final_scores(state: &GameState) {
    println!();
    println!("  ══ Final Scores ════════════════════════════════════");
    println!();
    let mut sorted: Vec<(usize, i32)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| (i, p.total_score))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    for (rank, (i, score)) in sorted.iter().enumerate() {
        let crown = if rank == 0 { " 👑" } else { "   " };
        println!("  {}. Player {i}{crown}  {score:4} pts", rank + 1);
    }
    println!();
}

// ── I/O helpers ───────────────────────────────────────────────────────────────

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf
}

fn read_usize(prompt: &str, min: usize, max: usize) -> usize {
    loop {
        let line = read_line(prompt);
        match line.trim().parse::<usize>() {
            Ok(n) if n >= min && n <= max => return n,
            _ => println!("  Enter a number between {min} and {max}."),
        }
    }
}

fn ask_yn(prompt: &str, default_yes: bool) -> bool {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        let line = read_line(&format!("{prompt} {hint} "));
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            "" => return default_yes,
            _ => println!("  Please answer y or n."),
        }
    }
}
