use crate::utils::BinaryStream;
use crate::protocol::packet::DataPacket;
use crate::protocol::info::ProtocolInfo;

pub const PERMISSION_NORMAL: u32 = 0;
pub const PERMISSION_OPERATOR: u32 = 1;
pub const PERMISSION_HOST: u32 = 2;
pub const PERMISSION_AUTOMATION: u32 = 3;
pub const PERMISSION_ADMIN: u32 = 4;

pub struct AdventureSettingsPacket {
    pub flags: u32,
    pub user_permission: u32,
    pub world_immu_table: bool,
    pub no_pvp: bool,
    pub no_pvm: bool,
    pub no_mvp: bool,
    pub auto_jump: bool,
    pub allow_flight: bool,
    pub no_clip: bool,
    pub world_builder: bool,
    pub is_flying: bool,
    pub muted: bool,
}

impl Default for AdventureSettingsPacket {
    fn default() -> Self { Self::new() }
}

impl AdventureSettingsPacket {
    pub fn new() -> Self {
        Self {
            flags: 0,
            user_permission: PERMISSION_NORMAL,
            world_immu_table: false,
            no_pvp: false,
            no_pvm: false,
            no_mvp: false,
            auto_jump: false,
            allow_flight: false,
            no_clip: false,
            world_builder: false,
            is_flying: false,
            muted: false,
        }
    }
}

impl DataPacket for AdventureSettingsPacket {
    const NETWORK_ID: u8 = ProtocolInfo::ADVENTURE_SETTINGS_PACKET;

    fn encode_payload(&self, stream: &mut BinaryStream) {
        let mut flags = 0u32;
        flags |= self.world_immu_table as u32;
        flags |= (self.no_pvp as u32) << 1;
        flags |= (self.no_pvm as u32) << 2;
        flags |= (self.no_mvp as u32) << 3;
        flags |= (self.auto_jump as u32) << 5;
        flags |= (self.allow_flight as u32) << 6;
        flags |= (self.no_clip as u32) << 7;
        flags |= (self.world_builder as u32) << 8;
        flags |= (self.is_flying as u32) << 9;
        flags |= (self.muted as u32) << 10;
        stream.put_unsigned_var_int(flags);
        stream.put_unsigned_var_int(self.user_permission);
    }

    fn decode(buf: Vec<u8>) -> Self where Self: Sized {
        let mut stream = BinaryStream::from_bytes(buf);
        let _ = stream.get_byte();
        let flags = stream.get_unsigned_var_int().unwrap_or(0);
        let user_permission = stream.get_unsigned_var_int().unwrap_or(0);
        Self {
            flags,
            user_permission,
            world_immu_table: (flags & 1) != 0,
            no_pvp: (flags & (1 << 1)) != 0,
            no_pvm: (flags & (1 << 2)) != 0,
            no_mvp: (flags & (1 << 3)) != 0,
            auto_jump: (flags & (1 << 5)) != 0,
            allow_flight: (flags & (1 << 6)) != 0,
            no_clip: (flags & (1 << 7)) != 0,
            world_builder: (flags & (1 << 8)) != 0,
            is_flying: (flags & (1 << 9)) != 0,
            muted: (flags & (1 << 10)) != 0,
        }
    }

    fn name(&self) -> &'static str { "AdventureSettingsPacket" }
}
