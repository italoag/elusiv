#![allow(clippy::large_enum_variant)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::system_program;
use solana_program::sysvar::instructions;
use elusiv_types::AccountRepr;
use crate::apa::{ApaProposal, ApaProposalAccount, ApaProposalsAccount, ApaTargetMapAccount};
use crate::network::BasicWardenNetworkAccount;
use crate::warden::{
    ElusivWardenID,
    ElusivBasicWardenConfig,
    WardensAccount,
    BasicWardenAccount,
    BasicWardenOperatorAccount,
    BasicWardenMapAccount,
    BasicWardenStatsAccount,
    BasicWardenAttesterMapAccount,
    Identifier,
    Timezone,
    WardenRegion,
};
use crate::macros::ElusivInstruction;
use crate::processor;

#[cfg(feature = "elusiv-client")]
pub use elusiv_types::accounts::{UserAccount, SignerAccount, WritableUserAccount, WritableSignerAccount};

#[repr(u8)]
#[derive(BorshDeserialize, BorshSerialize, ElusivInstruction)]
pub enum ElusivWardenNetworkInstruction {
    // -------- Program initialization --------

    #[acc(payer, { signer, writable })]
    #[pda(wardens, WardensAccount, { writable, find_pda, account_info })]
    #[pda(basic_network, BasicWardenNetworkAccount, { writable, find_pda, account_info })]
    #[pda(proposals_account, ApaProposalsAccount, { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    Init,

    #[acc(payer, { signer, writable })]
    #[pda(basic_network, BasicWardenNetworkAccount, pda_offset = Some(region.pda_offset()), { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    InitRegionAccount {
        region: WardenRegion,
    },

    // -------- Basic Warden --------

    #[acc(warden, { signer, writable })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable, find_pda, account_info })]
    #[pda(warden_map_account, BasicWardenMapAccount, pda_pubkey = config.key, { writable, find_pda, account_info })]
    #[pda(wardens, WardensAccount, { writable })]
    #[pda(basic_network, BasicWardenNetworkAccount, { writable })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    RegisterBasicWarden {
        warden_id: ElusivWardenID,
        config: ElusivBasicWardenConfig,
    },

    #[acc(operator, { signer, writable })]
    #[pda(operator_account, BasicWardenOperatorAccount, pda_pubkey = operator.pubkey(), { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    RegisterBasicWardenOperator {
        ident: Identifier,
        url: Identifier,
        jurisdiction: Option<u16>,
    },

    #[acc(operator, { signer })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    ConfirmBasicWardenOperation {
        warden_id: ElusivWardenID,
    },

    #[acc(warden, { signer })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    UpdateBasicWardenState {
        warden_id: ElusivWardenID,
        is_active: bool,
    },

    #[acc(warden, { signer })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    #[acc(lut_account)]
    UpdateBasicWardenLut {
        warden_id: ElusivWardenID,
    },

    // -------- Basic Warden statistics --------

    #[acc(warden)]
    #[acc(payer, { signer, writable })]
    #[pda(stats_account, BasicWardenStatsAccount, pda_pubkey = warden.pubkey(), pda_offset = Some(year.into()), { writable, find_pda, account_info })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    OpenBasicWardenStatsAccount {
        year: u16,
    },

    #[acc(warden)]
    #[pda(stats_account, BasicWardenStatsAccount, pda_pubkey = warden.pubkey(), pda_offset = Some(year.into()), { writable })]
    #[sys(instructions, key = instructions::ID)]
    TrackBasicWardenStats {
        year: u16,
        can_fail: bool,
    },

    // -------- APA --------

    #[acc(proponent, { signer, writable })]
    #[pda(proposal_account, ApaProposalAccount, pda_offset = Some(proposal_id), { writable, find_pda, account_info })]
    #[pda(proposals_account, ApaProposalsAccount, { writable })]
    #[pda(map_account, ApaTargetMapAccount, pda_pubkey = proposal.target, { writable, find_pda, account_info })]
    #[acc(token_mint)]
    #[sys(system_program, key = system_program::ID, { ignore })]
    ProposeApaProposal {
        proposal_id: u32,
        proposal: ApaProposal,
    },

    // -------- Metadata attestation --------

    #[acc(signer, { signer, writable })]
    #[acc(attester)]
    #[pda(attester_account, BasicWardenAttesterMapAccount, pda_pubkey = attester.pubkey(), { writable, find_pda, account_info })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    AddMetadataAttester {
        warden_id: ElusivWardenID,
    },

    #[acc(signer, { signer, writable })]
    #[acc(attester)]
    #[pda(attester_account, BasicWardenAttesterMapAccount, pda_pubkey = attester.pubkey(), { writable, account_info })]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    RevokeMetadataAttester {
        warden_id: ElusivWardenID,
    },

    #[acc(attester, { signer })]
    #[pda(attester_warden_account, BasicWardenAccount, pda_offset = Some(attester_warden_id))]
    #[pda(warden_account, BasicWardenAccount, pda_offset = Some(warden_id), { writable })]
    AttestBasicWardenMetadata {
        attester_warden_id: ElusivWardenID,
        warden_id: ElusivWardenID,
        asn: Option<u32>,
        timezone: Timezone,
        region: WardenRegion,
        uses_proxy: bool,
    },

    // -------- Program state management --------

    #[cfg(not(feature = "mainnet"))]
    #[acc(payer, { signer })]
    #[acc(recipient, { writable })]
    #[acc(program_account, { writable })]
    #[sys(system_program, key = system_program::ID, { ignore })]
    CloseProgramAccount,
}