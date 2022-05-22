use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use crate::bytes::SerDe;

/// This trait is used by the elusiv_instruction macro
pub trait PDAAccount {
    const SEED: &'static [u8];

    fn pubkey(offsets: &[u64]) -> (Pubkey, u8) {
        let seed = Self::offset_seed(offsets);
        let seed: Vec<&[u8]> = seed.iter().map(|x| &x[..]).collect();
        Pubkey::find_program_address(&seed, &crate::id())
    }

    fn offset_seed(offsets: &[u64]) -> Vec<Vec<u8>> {
        let mut seed: Vec<Vec<u8>> = vec![Self::SEED.to_vec()];
        let offsets: Vec<Vec<u8>> = offsets.iter().map(|x| x.to_le_bytes().to_vec()).collect();
        seed.extend(offsets);
        seed
    }

    fn is_valid_pubkey(offsets: &[u64], pubkey: &Pubkey) -> bool {
        Self::pubkey(offsets).0 == *pubkey
    }
} 

pub trait SizedAccount: PDAAccount {
    const SIZE: usize;
}

/// Certain accounts, like the `VerificationAccount` can be instantiated multiple times.
/// - this allows for parallel computations
/// - we use this trait to verify that a certain PDA is valid, since we generate PDAs with their index as last seed element
/// - so we can compare this index with `MAX_INSTANCES` to check validity
pub trait MultiInstanceAccount: PDAAccount {
    const MAX_INSTANCES: u64;

    fn is_valid(&self, index: u64) -> bool {
        index < Self::MAX_INSTANCES
    }
}

pub const MAX_ACCOUNT_SIZE: usize = 10_000_000;

/// Allows for storing data across multiple accounts
pub trait MultiAccountAccount<'t>: PDAAccount {
    /// The count of subsidiary accounts
    const COUNT: usize;
    fn get_account(&self, account_index: usize) -> &AccountInfo<'t>;
}

macro_rules! data_slice {
    ($data: ident, $index: ident) => {
        $data[$index * Self::T::SIZE..($index + 1) * Self::T::SIZE] 
    };
}

/// Allows for storing data in an array that cannot be stored in a single Solana account
/// - BigArrayAccount takes care of parsing the data stored in those accounts
/// - these accounts are PDA accounts generated by extending the BigArrayAccount's pda_seed
pub trait BigArrayAccount<'a>: MultiAccountAccount<'a> {
    type T: SerDe<T=Self::T>;

    const MAX_VALUES_PER_ACCOUNT: usize = MAX_ACCOUNT_SIZE / Self::T::SIZE;

    // indices in this implementation are always the external array indices and not byte-indices!
    fn account_and_local_index(&self, index: usize) -> (usize, usize) {
        let account_index = index / Self::MAX_VALUES_PER_ACCOUNT;
        (account_index, index - account_index * Self::MAX_VALUES_PER_ACCOUNT)
    }

    fn get(&self, index: usize) -> Self::T {
        let (account_index, local_index) = self.account_and_local_index(index);
        let account = self.get_account(account_index);
        let data = &account.data.borrow_mut()[..];
        Self::T::deserialize(&data_slice!(data, local_index))
    }

    fn set(&self, index: usize, value: Self::T) {
        let (account_index, local_index) = self.account_and_local_index(index);
        let account = self.get_account(account_index);
        let data = &mut account.data.borrow_mut()[..];
        Self::T::serialize(value, &mut data_slice!(data, local_index))
    }
}

pub const fn big_array_accounts_count(size: usize, element_size: usize) -> usize {
    let max = MAX_ACCOUNT_SIZE / element_size;
    size / max + (if size % max == 0 { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_set_big_array() {
        panic!()
    }
}