use rand::{rng, seq::SliceRandom};

use crate::game::cards::{ActionCard, Card, ModifierCard};

pub fn create_deck() -> Vec<Card> {
    let mut deck = Vec::new();

    for n in 0..=12 {
        for _ in 0..n.max(1) {
            deck.push(Card::Number(n));
        }
    }

    for _ in 0..3 {
        deck.push(Card::Action(ActionCard::Freeze));
        deck.push(Card::Action(ActionCard::Tap3));
        deck.push(Card::Action(ActionCard::Lifeline));
    }
    for value in [2, 4, 6, 8, 10] {
        deck.push(Card::Modifier(ModifierCard::Add(value)));
    }
    deck.push(Card::Modifier(ModifierCard::Multiply2));

    deck
}

pub fn shuffle_deck(deck: &mut Vec<Card>) {
    deck.shuffle(&mut rng());
}
