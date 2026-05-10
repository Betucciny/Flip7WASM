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
        let mode = pick_game_mode();
        let hints = ask_yn("  Show AI advisor hints?", true);
        let sim_count = if hints { pick_sim_count() } else { 500 };
        run_game(n, mode, hints, sim_count);
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

fn pick_sim_count() -> u32 {
    let items = [
        "Fast      ·    500 simulations",
        "Balanced  ·  2 000 simulations",
        "Accurate  ·  5 000 simulations",
        "Precise   · 10 000 simulations",
    ];
    let idx = Select::with_theme(&theme())
        .with_prompt("  Advisor quality")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);
    match idx {
        0 => 500,
        1 => 2_000,
        2 => 5_000,
        _ => 10_000,
    }
}

fn pick_game_mode() -> CliMode {
    let items = [
        "🎮  Simulator   engine draws random cards",
        "📋  Tracker     you enter each card  (real-game tracking)",
    ];
    let idx = Select::with_theme(&theme())
        .with_prompt("  Game mode")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);
    if idx == 0 {
        CliMode::Simulator
    } else {
        CliMode::Tracker
    }
}

// ── Game loop ─────────────────────────────────────────────────────────────────

fn run_game(player_count: usize, mode: CliMode, hints: bool, sim_count: u32) {
    let mut state = new_game(player_count);
    println!("\n  ▶  Game started — {} players\n", player_count);
    loop {
        run_round(&mut state, mode, hints, sim_count);
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

// ── Turn result ──────────────────────────────────────────────────────────────

enum Turn {
    Act(Action),
    Undo,
    Quit,
}

// ── Game mode ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum CliMode {
    /// Engine draws random cards from its shuffled deck.
    Simulator,
    /// Player enters each card drawn (real-game tracking).
    Tracker,
}

// ── Round loop ────────────────────────────────────────────────────────────────

fn run_round(state: &mut GameState, mode: CliMode, hints: bool, sim_count: u32) {
    let mut history: Vec<GameState> = Vec::new();
    clear_screen();
    print_round_header(state);

    while state.phase == GamePhase::Playing {
        clear_screen();
        print_round_header(state);
        print_board(state);
        print_player_detail(state);

        // Compute and display advisor recommendation only when hints are on.
        if hints {
            let rec = recommend(state, sim_count);
            if let Some(ref r) = rec {
                print_advice(r);
            }
        }

        match select_action(state, !history.is_empty(), mode) {
            Turn::Quit => {
                println!("\n  Exiting.\n");
                std::process::exit(0);
            }
            Turn::Undo => {
                if let Some(prev) = history.pop() {
                    *state = prev;
                    println!("\n  ↩  Undo — restored previous state.\n");
                    pause_ms(600);
                }
                // loop re-renders the board with the restored state
            }
            Turn::Act(action) => {
                let pid = state.current_player;
                let snapshot = state.clone();
                match apply_action(state.clone(), pid, action) {
                    Ok(new) => {
                        // save pre-action snapshot, evicting the oldest if full
                        if history.len() >= MAX_HISTORY {
                            history.remove(0);
                        }
                        history.push(snapshot);
                        *state = new;
                    }
                    Err(e) => {
                        println!("\n  ⚠  Engine error: {e:?}\n");
                        pause_ms(1500);
                    }
                }
            }
        }
    }
}

// ── Action selection ──────────────────────────────────────────────────────────

/// Returns the turn result: an action to apply, an undo request, or quit.
fn select_action(state: &GameState, can_undo: bool, mode: CliMode) -> Turn {
    // ── Forced: pending effect needs resolution ────────────────────────────
    if let Some(effect) = &state.pending_effect {
        // When undo is available, give the player a chance to back out
        // before being forced into the target-picker sub-prompts.
        if can_undo {
            let pre_items = ["⚡  Resolve effect", "↩  Undo last action", "🚪  Quit"];
            let choice = Select::with_theme(&theme())
                .with_prompt("  Pending effect — resolve or undo?")
                .items(&pre_items)
                .default(0)
                .interact()
                .unwrap_or(0);
            match choice {
                1 => return Turn::Undo,
                2 => return Turn::Quit,
                _ => {} // 0 → fall through to resolution
            }
        }

        return match effect {
            PendingEffect::Freeze => {
                let target = pick_target(state, "❄️  Freeze which player?");
                Turn::Act(Action::Freeze { target })
            }
            PendingEffect::Tap3 => {
                let target = pick_target(state, "👆  Tap3 which player?");
                if mode == CliMode::Simulator {
                    Turn::Act(Action::Tap3 { target })
                } else {
                    let cards = collect_tap3_cards(state, target);
                    Turn::Act(Action::Tap3Known { target, cards })
                }
            }
        };
    }

    // ── Normal turn ────────────────────────────────────────────────────────
    let draw_label = if mode == CliMode::Simulator {
        "🎲  Draw            draw a random card from the deck"
    } else {
        "🃏  Draw            enter the card you flipped"
    };
    let mut menu: Vec<&str> = vec![draw_label, "✋  Stop            lock in your current score"];
    if can_undo {
        menu.push("↩  Undo            restore the previous state");
    }
    menu.push("🚪  Quit");

    let quit_idx = menu.len() - 1;
    let undo_idx = quit_idx - 1; // only meaningful when can_undo is true

    loop {
        let i = Select::with_theme(&theme())
            .with_prompt("  What do you want to do?")
            .items(&menu)
            .default(0)
            .interact()
            .unwrap_or(usize::MAX);

        return match i {
            0 => {
                if mode == CliMode::Simulator {
                    Turn::Act(Action::Draw)
                } else {
                    println!();
                    Turn::Act(Action::DrawKnown { card: pick_card() })
                }
            }
            1 => Turn::Act(Action::Stop),
            _ if i == quit_idx => Turn::Quit,
            _ if can_undo && i == undo_idx => Turn::Undo,
            _ => continue,
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
    let targets: Vec<(usize, String)> = state
        .players
        .iter()
        .enumerate()
        .filter(|&(_, p)| p.status == PlayerStatus::Active)
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
const MAX_HISTORY: usize = 30;

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
