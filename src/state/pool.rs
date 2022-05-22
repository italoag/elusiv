//! Currently the single SOL pool used to store funds

use crate::macros::{pda_account, sized_account};

pub struct PoolAccount {}
pda_account!(PoolAccount, b"sol_pool");
sized_account!(PoolAccount, 1);