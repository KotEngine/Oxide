use crate::protocol::types::Item;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub action: TransactionAction,
    pub slot: usize,
    pub item: Item,
    pub source: TransactionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionAction {
    Take,
    Place,
    Swap,
    Drop,
    Click,
    ShiftClick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionSource {
    Player(String),
    System,
    Drop,
}

impl Transaction {
    pub fn new(slot: usize, item: Item, action: TransactionAction, source: TransactionSource) -> Self {
        Self { action, slot, item, source }
    }
}
