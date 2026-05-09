use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Card {
    Number(u8),
    Action(ActionCard),
    Modifier(ModifierCard),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionCard {
    Lifeline,
    Freeze,
    Tap3,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ModifierCard {
    Add(i32),
    Multiply2,
}
