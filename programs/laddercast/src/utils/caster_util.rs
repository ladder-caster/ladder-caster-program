use crate::account::Caster;
use crate::utils::{ItemRarity, MAX_LEVEL_0_BASED, RandomGenerator};

pub const EXPERIENCE_REQUIRED_PER_LEVEL: [u64; 30] = [502, 2000, 4985, 9950, 17387, 27789, 41648, 59457, 81707, 108891, 141502, 180032, 224973, 276818, 336059, 403189, 478700, 563084, 656833, 760441, 874399, 999200, 1135336, 1283300, 1443584, 1616681, 1803082, 2003280, 2217768, 2447038];

pub fn give_exp_to_caster_resources_burned(
    caster: &mut Caster,
    fire_burned: Option<u64>,
    earth_burned: Option<u64>,
    water_burned: Option<u64>,
) {
    //Give exp to user based on burned resources
    caster.experience +=
        fire_burned.unwrap_or(0) + earth_burned.unwrap_or(0) + water_burned.unwrap_or(0);


    //Since 0 based, we don't add +1 to level
    while caster.level <= MAX_LEVEL_0_BASED && caster.experience >= EXPERIENCE_REQUIRED_PER_LEVEL[(caster.level - 1) as usize] {
        caster.level += 1;
    }
}

pub fn give_exp_to_caster_spell(caster: &mut Caster, value: u64) {
    caster.experience += value;

    while caster.level <= MAX_LEVEL_0_BASED && caster.experience >= EXPERIENCE_REQUIRED_PER_LEVEL[(caster.level - 1) as usize] {
        caster.level += 1;
    }
}

pub fn is_spell_successful(rand: &mut RandomGenerator, spell_book_rarity: ItemRarity) -> bool {
    //Spell have a chance of working, they won't always work
    let max_range = match spell_book_rarity {
        ItemRarity::Common => 8,
        ItemRarity::Rare => 6,
        ItemRarity::Epic => 4,
        ItemRarity::Legendary => 2
    };

    rand.random_within_range::<u8, 1>(1, max_range) == 1
}