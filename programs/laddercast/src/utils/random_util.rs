use std::convert::TryInto;

use anchor_lang::solana_program::keccak::hashv;
use num_traits::Num;
use strum::EnumCount;

use crate::Pubkey;

pub struct RandomGenerator {
    offset: u8,
    hash: [u8; 32],
}

impl RandomGenerator {
    pub fn new(slot_hashes: &[u8], pubkey: Pubkey) -> Self {
        RandomGenerator {
            offset: 0,
            hash: hashv(&[&slot_hashes[8..64], &pubkey.to_bytes()]).to_bytes(),
        }
    }

    pub fn random<T: FromNE<N> + Num + Copy, const N: usize>(&mut self) -> T {
        self.offset += 1;
        T::from_ne_bytes(self.hash[self.offset as usize - 1..][..N].try_into().expect("Ran out of random"))
    }

    //Max is included as a potential number
    pub fn random_within_range<T: FromNE<N> + Num + Copy, const N: usize>(&mut self, min: T, max: T) -> T {
        self.random::<T, N>() % (max + T::one() - min) + min
    }

    pub fn random_enum<E: EnumCount + strum::IntoEnumIterator>(&mut self) -> E {
        let random_value: u8 = self.random_within_range(0, E::COUNT as u8 - 1);

        E::iter().nth(random_value as usize).unwrap()
    }

    pub fn random_enum_within_range<E: EnumCount + strum::IntoEnumIterator>(&mut self, min: u8, max: u8) -> E {
        let random_value: u8 = self.random_within_range(min, max);

        E::iter().nth(random_value as usize).unwrap()
    }
}

pub trait FromNE<const N: usize> {
    fn from_ne_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_from_ne_prim {
    (all $(($ty:ty, $size:expr)),+) => {
        $(impl_from_ne_prim!($ty, $size);)+
    };
    ($ty:ty, $size:expr) => {
        impl FromNE<$size> for $ty{
            fn from_ne_bytes(bytes: [u8; $size]) -> Self{
                Self::from_ne_bytes(bytes)
            }
        }
    }
}
impl_from_ne_prim!(all (u8, 1), (u16, 2), (u32, 4), (u64, 8), (u128, 16));