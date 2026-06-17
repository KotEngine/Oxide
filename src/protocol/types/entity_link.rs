#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityLinkType {
    Remove = 0,
    Rider = 1,
    Passenger = 2,
}

impl EntityLinkType {
    pub fn from_byte(b: u8) -> Self {
        match b {
            1 => Self::Rider,
            2 => Self::Passenger,
            _ => Self::Remove,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntityLink {
    pub from_entity_id: i64,
    pub to_entity_id: i64,
    pub link_type: EntityLinkType,
    pub immediate: bool,
}

impl EntityLink {
    pub fn new(from: i64, to: i64, link_type: EntityLinkType, immediate: bool) -> Self {
        Self { from_entity_id: from, to_entity_id: to, link_type, immediate }
    }
}
