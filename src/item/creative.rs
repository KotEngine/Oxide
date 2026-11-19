use std::sync::OnceLock;
use crate::protocol::types::Item;

static CREATIVE_ITEMS: OnceLock<Vec<Item>> = OnceLock::new();

#[derive(serde::Deserialize)]
struct RawEntry {
    id: i32,
    damage: i32,
    #[serde(default = "default_count")]
    count: u32,
}

fn default_count() -> u32 {
    1
}

pub fn get_creative_items() -> &'static [Item] {
    CREATIVE_ITEMS.get_or_init(|| {
        let raw = include_str!("../../resources/creativeitems.json");
        let list: Vec<RawEntry> = serde_json::from_str(raw).unwrap_or_default();
        list.into_iter().map(|e| Item::new(e.id, e.damage, e.count)).collect()
    })
}
