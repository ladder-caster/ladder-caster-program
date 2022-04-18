pub const FIRE_INDEX: usize = 0;
pub const WATER_INDEX: usize = 1;
pub const EARTH_INDEX: usize = 2;

//is number * 10 ^ 9 (1 000 000 000) since can only use u64 in transfer
pub const LADA_DISTRIBUTION_PER_TURN: u64 = 1_984_126_984_130;

pub const COST_IN_LADA_FOR_CASTER: u16 = 1_000;

pub const DECIMALS_PRECISION: u64 = 1_000_000_000;

//Game constants
pub const MAX_LEVEL_0_BASED: u8 = 29;
pub const MAX_LEVEL_1_BASED: u8 = 30;
pub const MAX_COLUMN_0_BASED: u8 = 2;
pub const CRAFTING_COST_MULTIPLIER: u8 = 5;
pub const MOVE_COST_MULTIPLIER: u8 = 10;

//Tile constants
pub const MINIMUM_TILE_LIFE: u8 = 3;

//Player constants
pub const DEFAULT_CRITICAL_CHANCE_IN_PERCENT: u16 = 200;
pub const DEFAULT_MAGIC_FIND_IN_PERCENT: u16 = 1000;

//NFT related
pub const NFT_MINT_DESCRIPTION: &str = "LC";
pub const NFT_CASTER_NAME: &str = "Caster";

//Actions related
pub const ACTION_LOOT_INDEX: usize = 0;
pub const ACTION_SPELL_INDEX: usize = 1;
pub const ACTION_MOVE_INDEX: usize = 2;
pub const ACTION_CRAFT_INDEX: usize = 3;

//Pub keys
pub const LADA_MINT_PUBKEY: &str = "95bzgMCtKw2dwaWufV9iZyu64DQo1eqw6QWnFMUSnsuF";
pub const LADA_ACCOUNT_PUBKEY: &str = "21XuJ9PZos9xYChGfVC4T9YKENc3UommR15qB7T6k6nN";
pub const GAME_CREATOR_AUTHORITY_PUBKEY: &str = "LCHZ3weMDr4ukhoEGHFMZGT4edLRN79L7foRMyTczA5";
pub const NFT_CREATOR_SPLITTER_PUBKEY: &str = "4HAz1eNba28njBhWKeVRUUn4tSobY1rNPP6MdUwMoBpa";
