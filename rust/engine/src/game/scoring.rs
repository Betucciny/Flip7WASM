use crate::game::{cards::ModifierCard, state::Player};

/// Computes the round score for a player.
///
/// Evaluation order:
/// 1. Sum all number cards.
/// 2. Apply all `Add` modifiers.
/// 3. Apply the `Multiply2` modifier (if present) — order of receipt doesn't matter.
/// 4. Add the +15 Flip7 bonus when the player holds 7+ unique number cards.
///    The bonus is intentionally applied *after* the multiplier so it is not doubled.
pub fn calculate_score(player: &Player) -> i32 {
    let mut score: i32 = player.numbers.iter().map(|&n| n as i32).sum();

    // Step 2: all additions first
    for modifier in &player.modifiers {
        if let ModifierCard::Add(v) = modifier {
            score += v;
        }
    }

    // Step 3: then any multiplication
    for modifier in &player.modifiers {
        if let ModifierCard::Multiply2 = modifier {
            score *= 2;
        }
    }

    // Step 4: Flip7 bonus — only unique NUMBER cards count, not modifiers
    if player.numbers.len() >= 7 {
        score += 15;
    }

    score
}
