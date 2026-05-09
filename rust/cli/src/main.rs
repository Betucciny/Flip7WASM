use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use std::io::Write;

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

// ── Theme ─────────────────────────────────────────────────────────────────────

fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    print_banner();
    loop {
        let n = pick_player_count();
        run_game(n);
        if !ask_yn("  Play again?", false) {
            break;
        }
    }
    println!("\n  Goodbye!\n");
}

// ── Player count ──────────────────────────────────────────────────────────────

fn pick_player_count() -> usize {
    println!();
    let items = [
        "2 players",
        "3 players",
        "4 players",
        "5 players",
        "6 players",
        "7 players",
        "8 players",
        "9 players",
    ];
    let idx = Select::with_theme(&theme())
        .with_prompt("  How many players?")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);
    idx + 2
}

// ── Game loop ─────────────────────────────────────────────────────────────────

fn run_game(player_count: usize) {
    let mut state = new_game(player_count);
    println!("\n  ▶  Game started — {} players\n", player_count);
    loop {
        run_round(&mut state);
        print_round_results(&state);
        end_round(&mut state);

        if let Some(w) = find_winner(&state) {
            println!("\n  🏆  Player {w} reached 200 points — GAME OVER!\n");
            break;
        }

        if !ask_yn(&format!("  Start round {}?", state.round), true) {
            break;
        }
    }
    print_final_scores(&state);
}

fn find_winner(state: &GameState) -> Option<usize> {
    state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.total_score >= 200)
        .max_by_key(|(_, p)| p.total_score)
        .map(|(i, _)| i)
}

// ── Round loop ────────────────────────────────────────────────────────────────

fn run_round(state: &mut GameState) {
    clear_screen();
    print_round_header(state);

    while state.phase == GamePhase::Playing {
        clear_screen();
        print_round_header(state);
        print_board(state);
        print_player_detail(state);

        // Compute recommendation once per frame (before the prompt).
        let rec = recommend(state, 500);
        if let Some(ref r) = rec {
            print_advice(r);
        }

        let Some(action) = select_action(state) else {
            println!("\n  Exiting.\n");
            std::process::exit(0);
        };

        let pid = state.current_player;
        match apply_action(state.clone(), pid, action) {
            Ok(new) => *state = new,
            Err(e) => {
                println!("\n  ⚠  Engine error: {e:?}\n");
                pause_ms(1500);
            }
        }
    }
}

// ── Action selection ──────────────────────────────────────────────────────────

/// Returns the action to apply, or `None` when the user chooses Quit.
fn select_action(state: &GameState) -> Option<Action> {
    // ── Forced: pending effect needs resolution ────────────────────────────
    if let Some(effect) = &state.pending_effect {
        return match effect {
            PendingEffect::Freeze => {
                let target = pick_target(state, "❄️  Freeze which player?");
                Some(Action::Freeze { target })
            }
            PendingEffect::Tap3 => {
                let mode_items = [
                    "🎲  Random draw  (simulation — unknown card)",
                    "🃏  Enter known cards  (real-game tracking)",
                ];
                let mode = Select::with_theme(&theme())
                    .with_prompt("  👆  Tap3 — how are you drawing?")
                    .items(&mode_items)
                    .default(0)
                    .interact()
                    .unwrap_or(0);

                let target = pick_target(state, "👆  Tap3 which player?");

                if mode == 0 {
                    Some(Action::Tap3 { target })
                } else {
                    let cards = collect_tap3_cards(state, target);
                    Some(Action::Tap3Known { target, cards })
                }
            }
        };
    }

    // ── Normal turn ────────────────────────────────────────────────────────
    let items = [
        "🎲  Draw            draw a random card from the deck",
        "✋  Stop            lock in your current score",
        "🚪  Quit",
    ];

    loop {
        let i = Select::with_theme(&theme())
            .with_prompt("  What do you want to do?")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(usize::MAX);

        return match i {
            0 => {
                println!();
                Some(Action::DrawKnown { card: pick_card() })
            }
            1 => Some(Action::Stop),
            2 => None,
            _ => continue, // shouldn't happen
        };
    }
}

// ── Card picker ───────────────────────────────────────────────────────────────

fn pick_card() -> Card {
    let type_items = [
        "Number      0 – 12",
        "Action      Lifeline / Freeze / Tap3",
        "Modifier    +2  +4  +6  +8  +10  ×2",
    ];

    let type_i = Select::with_theme(&theme())
        .with_prompt("  Card type")
        .items(&type_items)
        .default(0)
        .interact()
        .unwrap_or(0);

    match type_i {
        0 => {
            // All 13 number values as labelled strings.
            let nums: Vec<String> = (0u8..=12).map(|n| format!("  {n}")).collect();
            let n_i = Select::with_theme(&theme())
                .with_prompt("  Number card")
                .items(&nums)
                .default(0)
                .interact()
                .unwrap_or(0);
            Card::Number(n_i as u8)
        }
        1 => {
            let actions = ["🛡️  Lifeline", "❄️  Freeze", "👆  Tap3"];
            let a_i = Select::with_theme(&theme())
                .with_prompt("  Action card")
                .items(&actions)
                .default(0)
                .interact()
                .unwrap_or(0);
            match a_i {
                0 => Card::Action(ActionCard::Lifeline),
                1 => Card::Action(ActionCard::Freeze),
                _ => Card::Action(ActionCard::Tap3),
            }
        }
        _ => {
            let mods = ["+2", "+4", "+6", "+8", "+10", "×2  (Multiply 2)"];
            let m_i = Select::with_theme(&theme())
                .with_prompt("  Modifier card")
                .items(&mods)
                .default(0)
                .interact()
                .unwrap_or(0);
            match m_i {
                0 => Card::Modifier(ModifierCard::Add(2)),
                1 => Card::Modifier(ModifierCard::Add(4)),
                2 => Card::Modifier(ModifierCard::Add(6)),
                3 => Card::Modifier(ModifierCard::Add(8)),
                4 => Card::Modifier(ModifierCard::Add(10)),
                _ => Card::Modifier(ModifierCard::Multiply2),
            }
        }
    }
}

// ── Target picker ─────────────────────────────────────────────────────────────

fn pick_target(state: &GameState, prompt: &str) -> usize {
    let pid = state.current_player;
    let targets: Vec<(usize, String)> = state
        .players
        .iter()
        .enumerate()
        .filter(|&(i, p)| i != pid && p.status == PlayerStatus::Active)
        .map(|(i, p)| {
            let nums: Vec<String> = p.numbers.iter().map(|n| n.to_string()).collect();
            let label = format!(
                "Player {i}  ({} pts)  [{}]",
                calculate_score(p),
                if nums.is_empty() {
                    "—".into()
                } else {
                    nums.join(" ")
                }
            );
            (i, label)
        })
        .collect();

    let labels: Vec<&str> = targets.iter().map(|(_, s)| s.as_str()).collect();

    let i = Select::with_theme(&theme())
        .with_prompt(prompt)
        .items(&labels)
        .default(0)
        .interact()
        .unwrap_or(0);

    targets[i].0
}

// ── Tap3 card collector ───────────────────────────────────────────────────────

/// Interactively collects up to 3 cards the Tap3 victim actually drew,
/// stopping early on bust or Flip7.
fn collect_tap3_cards(state: &GameState, target: usize) -> Vec<Card> {
    let p = &state.players[target];
    let mut held: Vec<u8> = p.numbers.clone();
    let mut has_lifeline = p.has_lifeline;
    let mut cards: Vec<Card> = Vec::new();

    for draw_num in 1..=3usize {
        println!();
        println!("  ── P{target}: card {} of 3 ──", draw_num);
        let card = pick_card();

        // Simulate locally to detect bust / Flip7 and exit early.
        if let Card::Number(n) = &card {
            if held.contains(n) {
                if has_lifeline {
                    has_lifeline = false;
                    println!("  🛡️  Lifeline consumed — P{target} survives the duplicate.");
                } else {
                    cards.push(card);
                    println!("\n  💥  Player {target} busted on card {}!\n", draw_num);
                    pause_ms(800);
                    return cards;
                }
            } else {
                held.push(*n);
                if held.len() >= 7 {
                    cards.push(card);
                    println!("\n  🏅  Player {target} got FLIP 7!\n");
                    pause_ms(800);
                    return cards;
                }
            }
        }

        cards.push(card);

        // After cards 1 and 2, ask whether to enter another.
        if draw_num < 3
            && !ask_yn(
                &format!("  Enter card {} of 3 for P{target}?", draw_num + 1),
                true,
            )
        {
            break;
        }
    }

    cards
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    std::io::stdout().flush().unwrap();
}

fn pause_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

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
        let life = if p.has_lifeline { " 🛡" } else { "   " };
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

    // Chain banner — shown when resolving a queued effect from someone else's Tap3.
    if state.current_player != state.turn_holder {
        println!(
            "  ⛓  Chain: P{}'s Tap3 → P{pid} resolves a queued effect",
            state.turn_holder
        );
        println!();
    }

    // Pending-effect detail: list valid targets.
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
                    if nums.is_empty() {
                        "—".into()
                    } else {
                        nums.join(" ")
                    }
                );
            }
        }
        println!();
        return;
    }

    // Normal turn detail.
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

// ── Prompt helpers ────────────────────────────────────────────────────────────

fn ask_yn(prompt: &str, default: bool) -> bool {
    Confirm::with_theme(&theme())
        .with_prompt(prompt)
        .default(default)
        .interact()
        .unwrap_or(default)
}
