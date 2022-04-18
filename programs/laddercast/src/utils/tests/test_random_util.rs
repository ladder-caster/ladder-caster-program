#[cfg(test)]
mod test_internal_functions {
    use std::collections::HashMap;
    use std::convert::TryInto;

    use lazy_static::lazy_static;
    use rand::random;

    use crate::Pubkey;
    use crate::utils::RandomGenerator;

    lazy_static! {
        static ref SLOT_HASHES: [u8; 512 * 40] = generate_slot_hashes(true).try_into().unwrap();
    }

    fn generate_slot_hashes(is_random: bool) -> Vec<u8> {
        if is_random {
            (0..512 * 40).map(|_| random()).collect()
        } else {
            vec![1; 512 * 40]
        }
    }

    #[test]
    fn test_randomness_on_u8() {
        let mut hash_map: HashMap<u8, u8> = HashMap::new();

        let mut rand = RandomGenerator::new(SLOT_HASHES.as_slice(), Pubkey::new_unique());

        for _i in 0..20 {
            let random: u8 = rand.random::<u8, 1>();

            hash_map.entry(random).and_modify(|e| *e += 1).or_insert(1);
        }

        let mut total_number: u16 = 0;

        for value in hash_map.values() {
            //Since randomness, shouldn't be above 10
            assert!(*value <= 10);

            total_number += *value as u16;
        }

        assert_eq!(total_number, 20);
    }

    #[test]
    fn test_randomness_on_u8_with_range() {
        let mut hash_map: HashMap<u8, u8> = HashMap::new();

        let mut rand = RandomGenerator::new(SLOT_HASHES.as_slice(), Pubkey::new_unique());

        for _i in 0..20 {
            let random: u8 = rand.random_within_range::<u8, 1>(1, 2);

            hash_map.entry(random).and_modify(|e| *e += 1).or_insert(1);
        }

        let mut total_number: u16 = 0;

        for key in hash_map.keys() {
            //Since randomness with range, should always be between 1 and 2
            assert!(*key >= 1 && *key <= 2);

            total_number += *hash_map.get(key).unwrap() as u16;
        }

        assert_eq!(total_number, 20);
    }
}