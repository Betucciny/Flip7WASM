use engine::game::{
    cards::{ActionCard, Card, ModifierCard},
    deck::{create_deck, shuffle_deck},
    state::{Deck, GamePhase, GameState, Player, PlayerStatus},
};

// ─────────────────────────────────────────────────────────────────────────────

pub struct Scenario {
    pub name: String,
    pub description: String,
    pub state: GameState,
    pub player_id: usize,
}

/// Constructs a 4-player [`GameState`] where **player 0** holds the given
/// cards and the other three start with empty hands.
///
/// Every card that player 0 holds is removed from the draw pile so that all
/// probability calculations inside the simulation module are accurate.
fn make(
    name: impl Into<String>,
    description: impl Into<String>,
    numbers: Vec<u8>,
    modifiers: Vec<ModifierCard>,
    has_lifeline: bool,
) -> Scenario {
    const N_PLAYERS: usize = 4;

    let mut draw_pile = create_deck();

    // Remove each held number card (one copy per entry).
    for &n in &numbers {
        if let Some(pos) = draw_pile.iter().position(|c| c == &Card::Number(n)) {
            draw_pile.remove(pos);
        }
    }

    // Remove each held modifier card (one copy per entry).
    for m in &modifiers {
        let target = Card::Modifier(m.clone());
        if let Some(pos) = draw_pile.iter().position(|c| c == &target) {
            draw_pile.remove(pos);
        }
    }

    // Remove the lifeline from the deck if the player is already holding it.
    if has_lifeline {
        let target = Card::Action(ActionCard::Lifeline);
        if let Some(pos) = draw_pile.iter().position(|c| c == &target) {
            draw_pile.remove(pos);
        }
    }

    shuffle_deck(&mut draw_pile);

    let players = (0..N_PLAYERS)
        .map(|i| {
            if i == 0 {
                Player {
                    numbers: numbers.clone(),
                    modifiers: modifiers.clone(),
                    has_lifeline,
                    status: PlayerStatus::Active,
                    total_score: 0,
                }
            } else {
                Player {
                    numbers: vec![],
                    modifiers: vec![],
                    has_lifeline: false,
                    status: PlayerStatus::Active,
                    total_score: 0,
                }
            }
        })
        .collect();

    Scenario {
        name: name.into(),
        description: description.into(),
        state: GameState {
            players,
            deck: Deck {
                draw_pile,
                discard_pile: vec![],
            },
            current_player: 0,
            turn_holder: 0,
            round: 1,
            pending_effect: None,
            effect_queue: vec![],
            phase: GamePhase::Playing,
        },
        player_id: 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────

pub fn all_scenarios() -> Vec<Scenario> {
    vec![
        // ── Baseline ──────────────────────────────────────────────────────────
        make(
            "Empty hand",
            "No cards yet — first draw of the round. Pure upside.",
            vec![],
            vec![],
            false,
        ),
        // ── Low risk ─────────────────────────────────────────────────────────
        make(
            "3 low numbers [1,2,3]",
            "Safe early position. Very low bust risk, plenty of room to grow.",
            vec![1, 2, 3],
            vec![],
            false,
        ),
        make(
            "3 low numbers + Add10 [1,2,3]",
            "+10 modifier on a small hand bumps stop score to 16. Draw or bank it?",
            vec![1, 2, 3],
            vec![ModifierCard::Add(10)],
            false,
        ),
        // ── Medium risk ───────────────────────────────────────────────────────
        make(
            "5 medium numbers [2,4,6,8,10]",
            "Solid stop score of 30. Moderate bust risk — classic mid-game decision.",
            vec![2, 4, 6, 8, 10],
            vec![],
            false,
        ),
        make(
            "5 numbers + Add10 [3,5,7,9,11]",
            "Stop score is 45. High value hand — is the draw worth the risk?",
            vec![3, 5, 7, 9, 11],
            vec![ModifierCard::Add(10)],
            false,
        ),
        // ── High risk ────────────────────────────────────────────────────────
        make(
            "5 high numbers [6,7,8,9,10]",
            "Strong stop score of 40, but many duplicates remain. Very risky to draw.",
            vec![6, 7, 8, 9, 10],
            vec![],
            false,
        ),
        make(
            "5 high numbers + x2 [5,6,7,8,9]",
            "x2 modifier doubles a 35-pt hand to 70. Drawing risks a big bust.",
            vec![5, 6, 7, 8, 9],
            vec![ModifierCard::Multiply2],
            false,
        ),
        // ── Near Flip7 ───────────────────────────────────────────────────────
        make(
            "6 numbers near Flip7 [4,5,6,7,8,9]",
            "One card away from the +15 Flip7 bonus. Very high bust risk.",
            vec![4, 5, 6, 7, 8, 9],
            vec![],
            false,
        ),
        make(
            "6 numbers + lifeline [4,5,6,7,8,9]",
            "Same as above but the lifeline eliminates one bust. Much safer draw.",
            vec![4, 5, 6, 7, 8, 9],
            vec![],
            true,
        ),
        make(
            "6 numbers + x2 near Flip7 [3,4,5,6,7,8]",
            "x2 modifier on 6 numbers: stop=66, but Flip7 bonus isn't doubled. Worth it?",
            vec![3, 4, 5, 6, 7, 8],
            vec![ModifierCard::Multiply2],
            false,
        ),
    ]
}
