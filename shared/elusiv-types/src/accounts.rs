use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use elusiv_derive::BorshSerDeSized;
use crate::bytes::{BorshSerDeSized, ElusivOption};

pub trait SizedAccount {
    const SIZE: usize;
}

pub trait ProgramAccount<'a>: SizedAccount {
    type T: SizedAccount;

    fn new(d: &'a mut [u8]) -> Result<Self::T, ProgramError>;
}

pub trait MultiAccountProgramAccount<'a, 'b, 't> {
    type T: SizedAccount;

    fn new(
        d: &'a mut [u8],
        accounts: std::collections::HashMap<usize, &'b AccountInfo<'t>>,
    ) -> Result<Self::T, ProgramError>;
}

pub type PDAOffset = Option<u32>;

/// This trait is used by the `elusiv_instruction` and `elusiv_accounts` macros
/// - a PDAAccount is simply a PDA with:
///     1. the leading fields specified by `PDAAccountFields`
///     2. a PDA that is derived using the following seed: `&[ &SEED, offset?, bump ]`
/// - so there are two kinds of PDAAccounts:
///     - single instance: the `pda_offset` is `None` -> `&[ &SEED, bump ]`
///     - multi instance: the `pda_offset` is `Some(offset)` -> `&[ &SEED, offset, bump ]`
pub trait PDAAccount {
    const PROGRAM_ID: Pubkey;
    const SEED: &'static [u8];

    fn find(offset: PDAOffset) -> (Pubkey, u8) {
        let seed = Self::offset_seed(offset);
        let seed: Vec<&[u8]> = seed.iter().map(|x| &x[..]).collect();

        Pubkey::find_program_address(&seed, &Self::PROGRAM_ID)
    }

    fn pubkey(offset: PDAOffset, bump: u8) -> Result<Pubkey, ProgramError> {
        let mut seed = Self::offset_seed(offset);
        seed.push(vec![bump]);
        let seed: Vec<&[u8]> = seed.iter().map(|x| &x[..]).collect();

        match Pubkey::create_program_address(&seed, &Self::PROGRAM_ID) {
            Ok(v) => Ok(v),
            Err(_) => Err(ProgramError::InvalidSeeds)
        }
    }

    fn offset_seed(offset: PDAOffset) -> Vec<Vec<u8>> {
        match offset {
            Some(offset) => vec![Self::SEED.to_vec(), offset.to_le_bytes().to_vec()],
            None => vec![Self::SEED.to_vec()]
        }
    }

    fn is_valid_pubkey(account: &AccountInfo, offset: PDAOffset, pubkey: &Pubkey) -> Result<bool, ProgramError> {
        let bump = Self::get_bump(account);
        Ok(Self::pubkey(offset, bump)? == *pubkey)
    }

    fn get_bump(account: &AccountInfo) -> u8 {
        account.data.borrow()[0]
    }

    fn signers_seeds(pda_offset: PDAOffset, bump: u8) -> Vec<Vec<u8>> {
        match pda_offset {
            Some(pda_offset) => vec![
                Self::SEED.to_vec(),
                u32::to_le_bytes(pda_offset).to_vec(),
                vec![bump]
            ],
            None => vec![
                Self::SEED.to_vec(),
                vec![bump]
            ]
        }
    }
} 

#[derive(BorshDeserialize, BorshSerialize, BorshSerDeSized)]
pub struct PDAAccountData {
    pub bump_seed: u8,

    /// Used for future account migrations
    pub version: u8,

    /// In general useless, only if an account-type uses it
    pub initialized: bool,
}

impl PDAAccountData {
    pub fn new(data: &[u8]) -> Result<Self, std::io::Error> {
        PDAAccountData::try_from_slice(&data[..Self::SIZE])
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MultiAccountAccountData<const COUNT: usize> {
    // ... PDAAccountData always before MultiAccountAccountData, since it's a PDA
     
    pub pubkeys: [ElusivOption<Pubkey>; COUNT],
}

impl<const COUNT: usize> BorshSerDeSized for MultiAccountAccountData<COUNT> {
    const SIZE: usize = COUNT * <ElusivOption<Pubkey>>::SIZE;

    fn override_slice(value: &Self, slice: &mut [u8]) -> Result<(), std::io::Error> {
        let vec = Self::try_to_vec(value)?;
        slice[PDAAccountData::SIZE..PDAAccountData::SIZE + vec.len()].copy_from_slice(&vec[..]);
        Ok(())
    }
}

impl<const COUNT: usize> MultiAccountAccountData<COUNT> {
    pub fn new(data: &[u8]) -> Result<Self, std::io::Error> {
        MultiAccountAccountData::try_from_slice(&data[PDAAccountData::SIZE..PDAAccountData::SIZE + Self::SIZE])
    }
}

/// Certain accounts, like the `VerificationAccount` can be instantiated multiple times.
/// - this allows for parallel computations/usage
/// - so we can compare this index with `MAX_INSTANCES` to check validity
pub trait MultiInstancePDAAccount: PDAAccount {
    const MAX_INSTANCES: u64;

    fn is_valid(&self, index: u64) -> bool {
        index < Self::MAX_INSTANCES
    }
}

macro_rules! sub_account_safe {
    ($id: ident, $self: ident, $account_index: expr) => {
        let account = unsafe { $self.get_account_unsafe($account_index)? };
        let data = &mut account.data.borrow_mut()[..];
        let $id = SubAccount::new(data); 
    };
}

/// Allows for storing data across multiple accounts (needed for data sized >10 MiB)
/// - these accounts can be PDAs, but will most likely be data accounts (size > 10 KiB)
/// - by default all these accounts are assumed to have the same size = `ACCOUNT_SIZE`
/// - important: `ACCOUNT_SIZE` needs to contain `SUB_ACCOUNT_ADDITIONAL_SIZE`
pub trait MultiAccountAccount<'t>: PDAAccount {
    /// The count of subsidiary accounts
    const COUNT: usize;

    /// The size of subsidiary accounts
    const ACCOUNT_SIZE: usize;

    /// Finds all `n elem [0; COUNT]` available sub-accounts
    /// - the sub-accounts need to be supplied in correct order
    /// - any account that has been set (`pubkeys[i] == Some(_)`) can be used
    fn find_sub_accounts<'a, 'b, I, T, const COUNT: usize>(
        main_account: &'a AccountInfo<'b>,
        program_id: &Pubkey,
        writable: bool,
        account_info_iter: &mut I,
    ) -> Result<HashMap<usize, &'a AccountInfo<'b>>, ProgramError>
    where
        I: Iterator<Item = &'a AccountInfo<'b>> + Clone,
        T: PDAAccount + MultiAccountAccount<'b>,
    {
        assert_eq!(COUNT, Self::COUNT);

        let acc_data = &mut main_account.data.borrow_mut()[..];
        let fields_check = MultiAccountAccountData::<{COUNT}>::new(acc_data).or(Err(ProgramError::InvalidArgument))?;

        let mut accounts = HashMap::new();
        let mut remaining_iter = account_info_iter.clone();
        let mut i = 0;
        while i < Self::COUNT {
            match next_account_info(account_info_iter) {
                Ok(account) => {
                    for j in i..Self::COUNT {
                        match fields_check.pubkeys[j].option() {
                            Some(pk) => if *account.key != pk { continue },
                            None => continue,
                        }

                        if account.owner != program_id {
                            return Err(ProgramError::IllegalOwner);
                        }
                        if writable && !account.is_writable {
                            return Err(ProgramError::InvalidArgument)
                        }

                        accounts.insert(j, account);
                        next_account_info(&mut remaining_iter)?;
                        i = j;
                        break;
                    }
                    i += 1;
                }
                Err(_) => break,
            }
        }

        *account_info_iter = remaining_iter;
        Ok(accounts)
    }

    /// Returns the sub-account for the specified index
    /// 
    /// # Safety
    /// - Each sub-account has to be serialized using the `SubAccount` struct.
    /// - Modifiying/accessing without the `SubAccount` struct, can lead to undefined behaviour.
    /// - Use `execute_on_sub_account` instead of `get_account_unsafe` directly.
    unsafe fn get_account_unsafe(&self, account_index: usize) -> Result<&AccountInfo<'t>, ProgramError>;

    /// Ensures that the fields of `SubAccount` are not manipulated on a sub-account
    fn try_execute_on_sub_account<F, T, E>(&self, account_index: usize, f: F) -> Result<T, ProgramError> where F: Fn(&mut [u8]) -> Result<T, E> {
        sub_account_safe!(account, self, account_index);
        f(account.data).or(Err(ProgramError::InvalidAccountData))
    }

    fn execute_on_sub_account<F, T>(&self, account_index: usize, f: F) -> Result<T, ProgramError> where F: Fn(&mut [u8]) -> T {
        sub_account_safe!(account, self, account_index);
        Ok(f(account.data))
    }
}

/// Size required for the `is_in_use` boolean
pub const SUB_ACCOUNT_ADDITIONAL_SIZE: usize = 1;

pub struct SubAccount<'a> {
    is_in_use: &'a mut [u8],
    pub data: &'a mut [u8],
}

impl<'a> SubAccount<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        let (is_in_use, data) = data.split_at_mut(1);
        Self { is_in_use, data }
    }

    pub fn get_is_in_use(&self) -> bool {
        self.is_in_use[0] == 1
    }
    pub fn set_is_in_use(&mut self, value: bool) {
        self.is_in_use[0] = if value { 1 } else { 0 };
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestPDAAccount { }

    impl PDAAccount for TestPDAAccount {
        const PROGRAM_ID: Pubkey = Pubkey::new_unique();
        const SEED: &'static [u8] = b"ABC";
    }

    #[test]
    fn test_pda_account() {
        assert_ne!(TestPDAAccount::find(None), TestPDAAccount::find(Some(0)));
        assert_ne!(TestPDAAccount::find(Some(0)), TestPDAAccount::find(Some(1)));
    }

    #[test]
    fn test_sub_account() {
        let mut data = vec![0; 100];
        let mut account = SubAccount::new(&mut data);

        assert!(!account.get_is_in_use());
        account.set_is_in_use(true);
        assert!(account.get_is_in_use());
        account.set_is_in_use(false);
        assert!(!account.get_is_in_use());

        assert_eq!(account.data.len(), 99);
    }

    struct TestMultiAccount<'a, 'b> {
        pub pubkeys: [ElusivOption<Pubkey>; SUB_ACCOUNT_COUNT],
        pub accounts: std::collections::HashMap<usize, &'a AccountInfo<'b>>,
    }

    impl<'a, 'b> PDAAccount for TestMultiAccount<'a, 'b> {
        const PROGRAM_ID: Pubkey = Pubkey::new_unique();
        const SEED: &'static [u8] = b"ABC";
    }

    impl<'a, 'b> MultiAccountAccount<'b> for TestMultiAccount<'a, 'b> {
        const COUNT: usize = SUB_ACCOUNT_COUNT;
        const ACCOUNT_SIZE: usize = 2;

        unsafe fn get_account_unsafe(&self, account_index: usize) -> Result<&AccountInfo<'b>, ProgramError> {
            Ok(self.accounts[&account_index])
        }
    }

    impl<'a, 'b> TestMultiAccount<'a, 'b> {
        fn serialize(&self) -> Vec<u8> {
            let mut v = Vec::new();
            v.extend(vec![0; 3]);
            v.extend(self.pubkeys.try_to_vec().unwrap());
            v
        }
    }

    const SUB_ACCOUNT_COUNT: usize = 3;

    macro_rules! test_multi_account {
        ($accounts: ident, $pubkeys: ident) => {
            let mut accounts = HashMap::new();
            let mut pubkeys = [ElusivOption::None; SUB_ACCOUNT_COUNT];

            repeat!({
                let pk = solana_program::pubkey::Pubkey::new_unique();
                account!(account_index, pk, vec![1, 0]);
                accounts.insert(_index, &account_index);

                pubkeys[_index] = ElusivOption::Some(pk);
            }, 3);

            let $accounts = accounts;
            let $pubkeys = pubkeys;

        };

        ($id: ident) => {
            test_multi_account!(accounts, pubkeys);
            let $id = TestMultiAccount { pubkeys, accounts };
        };
        (mut $id: ident) => {
            test_multi_account!(accounts, pubkeys);
            let mut $id = TestMultiAccount { pubkeys, accounts };
        };
    }

    #[test]
    #[should_panic]
    fn test_get_account_unsafe() {
        test_multi_account!(account);
        unsafe { _ = account.get_account_unsafe(3); }
    }

    #[test]
    fn test_try_execute_on_sub_account() {
        test_multi_account!(account);

        for i in 0..SUB_ACCOUNT_COUNT {
            assert_eq!(
                account.try_execute_on_sub_account::<_, usize, ProgramError>(i, |data| {
                    data[0] = i as u8 + 1;
                    Ok(42)
                }).unwrap(),
                42
            );
        }

        for i in 0..SUB_ACCOUNT_COUNT {
            assert_eq!(account.accounts[&i].data.borrow()[1], i as u8 + 1);
        }
    }

    #[test]
    fn test_execute_on_sub_account() {
        test_multi_account!(account);

        for i in 0..SUB_ACCOUNT_COUNT {
            account.execute_on_sub_account(i, |data| {
                data[0] = i as u8 + 1;
            }).unwrap();
        }

        for i in 0..SUB_ACCOUNT_COUNT {
            assert_eq!(account.accounts[&i].data.borrow()[1], i as u8 + 1);
        }
    }

    fn test_find(
        pubkey_is_setup: [bool; SUB_ACCOUNT_COUNT],
        accounts: Vec<Option<usize>>,
        expected: Vec<usize>,
    ) {
        test_multi_account!(mut account);
        for (i, &is_setup) in pubkey_is_setup.iter().enumerate() {
            if is_setup { continue }
            account.pubkeys[i] = ElusivOption::None;
        }

        let data = account.serialize();
        let pk = solana_program::pubkey::Pubkey::new_unique();
        account!(main_account, pk, data);

        let pk = solana_program::pubkey::Pubkey::new_unique();
        account!(unused_account, pk, vec![1, 0]);

        let account_info_iter = &mut accounts.iter().map(|a| match a {
            Some(i) => account.accounts[i],
            None => &unused_account,
        });
        let len_prev = account_info_iter.len();
        let map = TestMultiAccount::find_sub_accounts::<_, TestMultiAccount, {SUB_ACCOUNT_COUNT}>(
            &main_account,
            &crate::ID,
            false,
            account_info_iter,
        ).unwrap();
        assert_eq!(len_prev, account_info_iter.len() + map.len());

        let mut keys: Vec<usize> = map.iter().map(|(&k, _)| k).collect();
        keys.sort_unstable();
        assert_eq!(keys, expected);
        for key in keys {
            assert_eq!(map[&key].key, account.accounts[&key].key);
        }
    }

    #[test]
    fn test_find_sub_accounts() {
        // All none
        test_find(
            [false, false, false],
            vec![Some(0), Some(1), Some(2)],
            vec![]
        );

        // First account set
        test_find(
            [true, false, false],
            vec![Some(0), Some(1), Some(2)],
            vec![0]
        );

        // Middle account set
        test_find(
            [false, true, false],
            vec![Some(0), Some(1), Some(2)],
            vec![1]
        );

        // Last account set
        test_find(
            [false, false, true],
            vec![Some(0), Some(1), Some(2)],
            vec![2]
        );

        // Different account at start
        test_find(
            [true, true, true],
            vec![None, Some(0), Some(1), Some(2)],
            vec![]
        );

        // Wrong order
        test_find(
            [true, true, true],
            vec![Some(2), Some(1), Some(0)],
            vec![2]
        );

        // Correct order
        test_find(
            [true, true, true],
            vec![Some(0), Some(1), Some(2)],
            vec![0, 1, 2]
        );

        // Accounts at end ignored
        test_find(
            [true, true, true],
            vec![Some(0), Some(1), Some(2), None, None],
            vec![0, 1, 2]
        );
    }
}*/