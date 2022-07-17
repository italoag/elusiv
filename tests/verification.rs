//! Tests the proof verification

#[cfg(not(tarpaulin_include))]
mod common;

use std::collections::HashMap;

use ark_ec::{ProjectiveCurve, PairingEngine};
use assert_matches::assert_matches;
use borsh::BorshSerialize;
use common::*;
use common::program_setup::*;
use elusiv::bytes::ElusivOption;
use elusiv::fields::u256_to_fr_skip_mr;
use elusiv::instruction::{ElusivInstruction, WritableUserAccount, SignerAccount, WritableSignerAccount, UserAccount};
use elusiv::proof::vkey::{VerificationKey, SendQuadraVKey};
use elusiv::proof::{VerificationAccount, VerificationState, PendingNullifierHashesMap, prepare_public_inputs_instructions, COMBINED_MILLER_LOOP_IXS, FINAL_EXPONENTIATION_IXS};
use elusiv::state::governor::{FeeCollectorAccount, GovernorAccount, FEE_COLLECTOR_MINIMUM_BALANCE, PoolAccount};
use elusiv::state::{MT_COMMITMENT_COUNT, StorageAccount, empty_root_raw};
use elusiv::state::program_account::{PDAAccount, ProgramAccount, MultiAccountProgramAccount};
use elusiv::types::{RawU256, Proof, SendPublicInputs, JoinSplitPublicInputs, U256Limbed2, PublicInputs};
use elusiv::proof::verifier::proof_from_str;
use elusiv::processor::ProofRequest;
use elusiv_utils::batch_instructions;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signer::Signer;

async fn setup_verification_tests() -> (ProgramTestContext, Actor) {
    let mut context = start_program_solana_program_test().await;

    setup_initial_accounts(&mut context).await;
    setup_storage_account(&mut context).await;
    create_merkle_tree(&mut context, 0).await;
    create_merkle_tree(&mut context, 1).await;

    let fee_collector = FeeCollectorAccount::find(None).0;
    airdrop(&fee_collector, FEE_COLLECTOR_MINIMUM_BALANCE, &mut context).await;

    let client = Actor::new(&mut context).await;
    (context, client)
}

#[derive(Clone)]
struct FullSendRequest {
    proof: Proof,
    public_inputs: SendPublicInputs,
}

fn send_requests() -> Vec<FullSendRequest> {
    vec![
        FullSendRequest {
            proof: proof_from_str(
                (
                    "10026859857882131638516328056627849627085232677511724829502598764489185541935",
                    "19685960310506634721912121951341598678325833230508240750559904196809564625591",
                    false,
                ),
                (
                    (
                        "857882131638516328056627849627085232677511724829502598764489185541935",
                        "685960310506634721912121951341598678325833230508240750559904196809564625591",
                    ),
                    (
                        "837064132573119120838379738103457054645361649757131991036638108422638197362",
                        "86803555845400161937398579081414146527572885637089779856221229551142844794",
                    ),
                        false,
                ),
                (
                    "21186803555845400161937398579081414146527572885637089779856221229551142844794",
                    "85960310506634721912121951341598678325833230508240750559904196809564625591",
                    false,
                ),
            ),
            public_inputs: SendPublicInputs {
                join_split: JoinSplitPublicInputs {
                    commitment_count: 1,
                    roots: vec![
                        Some(empty_root_raw()),
                    ],
                    nullifier_hashes: vec![
                        RawU256::new(u256_from_str_skip_mr("10026859857882131638516328056627849627085232677511724829502598764489185541935")),
                    ],
                    commitment: RawU256::new(u256_from_str_skip_mr("685960310506634721912121951341598678325833230508240750559904196809564625591")),
                    fee_version: 0,
                    amount: LAMPORTS_PER_SOL * 123,
                },
                recipient: RawU256::new(u256_from_str_skip_mr("19685960310506634721912121951341598678325833230508240750559904196809564625591")),
                current_time: 0,
                identifier: RawU256::new(u256_from_str_skip_mr("139214303935475888711984321184227760578793579443975701453971046059378311483")),
                salt: RawU256::new(u256_from_str_skip_mr("230508240750559904196809564625")),
            }
        },
        FullSendRequest {
            proof: proof_from_str(
                (
                    "10026859857882131638516328056627849627085232677511724829502598764489185541935",
                    "19685960310506634721912121951341598678325833230508240750559904196809564625591",
                    false,
                ),
                (
                    (
                        "857882131638516328056627849627085232677511724829502598764489185541935",
                        "685960310506634721912121951341598678325833230508240750559904196809564625591",
                    ),
                    (
                        "837064132573119120838379738103457054645361649757131991036638108422638197362",
                        "86803555845400161937398579081414146527572885637089779856221229551142844794",
                    ),
                    false,
                ),
                (
                    "21186803555845400161937398579081414146527572885637089779856221229551142844794",
                    "85960310506634721912121951341598678325833230508240750559904196809564625591",
                    false,
                ),
            ),
            public_inputs: SendPublicInputs {
                join_split: JoinSplitPublicInputs {
                    commitment_count: 2,
                    roots: vec![
                        Some(empty_root_raw()),
                        Some(empty_root_raw()),
                    ],
                    nullifier_hashes: vec![
                        RawU256::new(u256_from_str_skip_mr("10026859857882131638516328056627849627085232677511724829502598764489185541935")),
                        RawU256::new(u256_from_str_skip_mr("19685960310506634721912121951341598678325833230508240750559904196809564625591")),
                    ],
                    commitment: RawU256::new(u256_from_str_skip_mr("685960310506634721912121951341598678325833230508240750559904196809564625591")),
                    fee_version: 0,
                    amount: LAMPORTS_PER_SOL * 123,
                },
                recipient: RawU256::new(u256_from_str_skip_mr("19685960310506634721912121951341598678325833230508240750559904196809564625591")),
                current_time: 0,
                identifier: RawU256::new(u256_from_str_skip_mr("139214303935475888711984321184227760578793579443975701453971046059378311483")),
                salt: RawU256::new(u256_from_str_skip_mr("230508240750559904196809564625")),
            }
        },
    ]
}

#[tokio::test]
async fn test_verify_invalid_proof() {
    let (mut context, mut client) = setup_verification_tests().await;
    let (_, nullifier_0, _writable_nullifier_0) = nullifier_accounts(0, &mut context).await;
    let pending_nullifiers_map_account = pending_nullifiers_map_account(0, &mut context).await;
    let request = &send_requests()[0];

    pda_account!(governor, GovernorAccount, None, context);
    let fee = governor.get_program_fee();

    let fee_collector = FeeCollectorAccount::find(None).0;
    airdrop(&fee_collector, fee.base_commitment_subvention, &mut context).await;
    let fee_collector_balance = get_balance(&fee_collector, &mut context).await;

    let pool = PoolAccount::find(None).0;
    let pool_balance = get_balance(&pool, &mut context).await;

    // Init start
    ix_should_succeed(
        ElusivInstruction::init_verification_instruction(
            0,
            [0, 1],
            ProofRequest::Send(request.public_inputs.clone()),
            WritableSignerAccount(client.pubkey),
            &nullifier_0,
            &[],
        ),
        &mut client, &mut context,
    ).await;

    pda_account!(verification_account, VerificationAccount, Some(0), context);
    assert_matches!(verification_account.get_state(), VerificationState::None);
    let prepare_inputs_ix_count = verification_account.get_prepare_inputs_instructions_count();
    let public_inputs = request.public_inputs.public_signals_big_integer_skip_mr();
    let expected_instructions = prepare_public_inputs_instructions::<SendQuadraVKey>(&public_inputs);
    assert_eq!(expected_instructions.len() as u32, prepare_inputs_ix_count);
    for (i, &public_input) in public_inputs.iter().enumerate() {
        assert_eq!(verification_account.get_public_input(i).0, public_input);
    }

    // Subvention paid by fee_collector into pool
    assert_eq!(fee_collector_balance - fee.proof_subvention, get_balance(&fee_collector, &mut context).await);
    assert_eq!(pool_balance + fee.proof_subvention, get_balance(&pool, &mut context).await);

    // Init nullifiers
    ix_should_succeed(
        ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
            0,
            [0, 1],
            false,
            &[WritableUserAccount(pending_nullifiers_map_account)],
            &[],
        ),
        &mut client, &mut context,
    ).await;

    pda_account!(verification_account, VerificationAccount, Some(0), context);
    assert_matches!(verification_account.get_state(), VerificationState::NullifiersChecked);

    // Init proof
    ix_should_succeed(
        ElusivInstruction::init_verification_proof_instruction(
            0,
            request.proof.try_into().unwrap(),
            SignerAccount(client.pubkey),
        ),
        &mut client, &mut context,
    ).await;

    pda_account!(mut verification_account, VerificationAccount, Some(0), context);
    assert_matches!(verification_account.get_state(), VerificationState::ProofSetup);
    assert_eq!(verification_account.a.get().0, request.proof.a.0);
    assert_eq!(verification_account.b.get().0, request.proof.b.0);
    assert_eq!(verification_account.c.get().0, request.proof.c.0);
    assert_eq!(verification_account.get_vkey(), 0);

    // Input preparation
    for _ in 0..prepare_inputs_ix_count as u64 {
        tx_should_succeed(&[
            request_compute_units(1_400_000),
            ElusivInstruction::compute_verification_instruction(0)
        ], &mut client, &mut context).await;
    }

    // Check prepared inputs
    pda_account!(mut verification_account, VerificationAccount, Some(0), context);
    let public_inputs: Vec<ark_bn254::Fr> = request.public_inputs.public_signals().iter().map(|x| u256_to_fr_skip_mr(&x.reduce())).collect();
    let pvk = ark_pvk::<SendQuadraVKey>();
    let prepared_inputs = ark_groth16::prepare_inputs(&pvk, &public_inputs).unwrap().into_affine();
    assert_eq!(verification_account.prepared_inputs.get().0, prepared_inputs);

    // Combined miller loop
    let ix = ElusivInstruction::compute_verification_instruction(0);
    for ixs in batch_instructions(COMBINED_MILLER_LOOP_IXS, 350_000, ix.clone()) {
        tx_should_succeed(&ixs, &mut client, &mut context).await;
    }

    pda_account!(mut verification_account, VerificationAccount, Some(0), context);
    let combined_miller_loop_result = ark_bn254::Bn254::miller_loop(
        [
            (request.proof.a.0.into(), request.proof.b.0.into()),
            (prepared_inputs.into(), pvk.gamma_g2_neg_pc),
            (request.proof.c.0.into(), pvk.delta_g2_neg_pc),
        ]
        .iter(),
    );
    assert_eq!(verification_account.get_coeff_index(), 91);
    assert_eq!(verification_account.f.get().0, combined_miller_loop_result);

    // Final exponentiation
    for ixs in batch_instructions(FINAL_EXPONENTIATION_IXS, 1_400_000, ix.clone()) {
        tx_should_succeed(&ixs, &mut client, &mut context).await;
    }

    pda_account!(mut verification_account, VerificationAccount, Some(0), context);
    let final_exponentiation_result = ark_bn254::Bn254::final_exponentiation(&combined_miller_loop_result);
    assert_eq!(verification_account.f.get().0, final_exponentiation_result.unwrap());
    assert_matches!(verification_account.get_is_verified().option(), Some(false));

    let recipient = create_account(&mut context).await;
    let identifier = Pubkey::new(&request.public_inputs.identifier.skip_mr());
    let salt = Pubkey::new(&request.public_inputs.salt.skip_mr());

    let verification_account = VerificationAccount::find(Some(0)).0;
    let fee_collector = FeeCollectorAccount::find(None).0;
    let fee_collector_balance = get_balance(&fee_collector, &mut context).await;
    let rent = get_balance(&verification_account, &mut context).await;

    // Finalize
    tx_should_succeed(
        &[
            ElusivInstruction::finalize_verification_send_nullifiers_instruction(
                0,
                UserAccount(identifier),
                UserAccount(salt),
                Some(0),
                &[],
                Some(1),
                &[],
            ),
            ElusivInstruction::finalize_verification_payment_instruction(
                0,
                0,
                WritableUserAccount(recipient.pubkey()),
                WritableUserAccount(client.pubkey),
            ),
        ],
        &mut client, &mut context
    ).await;

    // VerificationAccount closed
    assert!(account_does_not_exist(verification_account, &mut context).await);

    // Subvention and rent transferred to fee_collector
    assert_eq!(
        fee_collector_balance + rent + fee.proof_subvention,
        get_balance(&fee_collector, &mut context).await
    );
    assert_eq!(pool_balance, get_balance(&pool, &mut context).await);
}

#[tokio::test]
async fn test_verify_valid_proof() {
    // TODO: proof is not actually valid, we just fake it later. Use actual valid proof and storage account instead

    let (mut context, mut client) = setup_verification_tests().await;
    let (_, nullifier_0, writable_nullifier_0) = nullifier_accounts(0, &mut context).await;
    let pending_nullifiers_map_account = pending_nullifiers_map_account(0, &mut context).await;
    let request = &send_requests()[0];

    pda_account!(governor, GovernorAccount, None, context);
    let fee = governor.get_program_fee();

    let fee_collector = FeeCollectorAccount::find(None).0;
    airdrop(&fee_collector, fee.base_commitment_subvention, &mut context).await;
    let fee_collector_balance = get_balance(&fee_collector, &mut context).await;

    let pool = PoolAccount::find(None).0;
    let pool_balance = get_balance(&pool, &mut context).await;

    // Init 
    tx_should_succeed(
        &[
            ElusivInstruction::init_verification_instruction(
                0,
                [0, 1],
                ProofRequest::Send(request.public_inputs.clone()),
                WritableSignerAccount(client.pubkey),
                &nullifier_0,
                &[],
            ),
            ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
                0,
                [0, 1],
                false,
                &[WritableUserAccount(pending_nullifiers_map_account)],
                &[],
            ),
            ElusivInstruction::init_verification_proof_instruction(
                0,
                request.proof.try_into().unwrap(),
                SignerAccount(client.pubkey),
            ),
        ],
        &mut client, &mut context,
    ).await;

    // Subvention paid by fee_collector into pool
    assert_eq!(fee_collector_balance - fee.proof_subvention, get_balance(&fee_collector, &mut context).await);
    assert_eq!(pool_balance + fee.proof_subvention, get_balance(&pool, &mut context).await);

    // Input preparation
    pda_account!(verification_account, VerificationAccount, Some(0), context);
    let prepare_inputs_ix_count = verification_account.get_prepare_inputs_instructions_count();
    for _ in 0..prepare_inputs_ix_count as u64 {
        tx_should_succeed(&[
            request_compute_units(1_400_000),
            ElusivInstruction::compute_verification_instruction(0)
        ], &mut client, &mut context).await;
    }

    // Combined miller loop
    let ix = ElusivInstruction::compute_verification_instruction(0);
    for ixs in batch_instructions(COMBINED_MILLER_LOOP_IXS, 350_000, ix.clone()) {
        tx_should_succeed(&ixs, &mut client, &mut context).await;
    }

    // Final exponentiation
    for ixs in batch_instructions(FINAL_EXPONENTIATION_IXS, 1_400_000, ix.clone()) {
        tx_should_succeed(&ixs, &mut client, &mut context).await;
    }

    // Fake valid proof (TODO: remove this)
    pda_account!(verification_account, VerificationAccount, Some(0), context);
    assert_matches!(verification_account.get_is_verified().option(), Some(false));
    set_pda_account::<VerificationAccount, _>(&mut context, Some(0), |data| {
        let mut verification_account = VerificationAccount::new(data).unwrap();
        verification_account.set_is_verified(&ElusivOption::Some(true));
    }).await;

    let recipient = Pubkey::new(&request.public_inputs.recipient.skip_mr());
    let identifier = Pubkey::new(&request.public_inputs.identifier.skip_mr());
    let salt = Pubkey::new(&request.public_inputs.salt.skip_mr());

    let pool = PoolAccount::find(None).0;
    airdrop(&pool, request.public_inputs.join_split.amount + LAMPORTS_PER_SOL * 10, &mut context).await;

    // Finalize
    tx_should_succeed(
        &[
            ElusivInstruction::finalize_verification_send_nullifiers_instruction(
                0,
                UserAccount(identifier),
                UserAccount(salt),
                Some(0),
                &[WritableUserAccount(writable_nullifier_0[0].0)],
                Some(1),
                &[],
            ),
            ElusivInstruction::finalize_verification_pending_nullifiers_instruction(
                0,
                Some(0),
                &[WritableUserAccount(pending_nullifiers_map_account)],
                Some(1),
                &[],
            ),
            ElusivInstruction::finalize_verification_payment_instruction(
                0,
                0,
                WritableUserAccount(recipient),
                WritableUserAccount(client.pubkey),
            ),
        ],
        &mut client, &mut context
    ).await;

    // TODO: check that nullifiers are actually added

    // Fee and rent paid to fee_payer

    // Funds sent to recipient
    let amount = request.public_inputs.join_split.amount;
    let subvention = fee.proof_subvention;
    let fee = fee.proof_verification_fee(prepare_inputs_ix_count as usize, 0, amount);
    assert_eq!(amount + subvention - fee, get_balance(&recipient, &mut context).await);

    // VerificationAccount closed
    //assert!(account_does_not_exist(verification_account, &mut context).await);

}

fn ark_pvk<VKey: VerificationKey>() -> ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254> {
    let mut gamma_abc_g1 = Vec::new();
    for i in 0..=VKey::PUBLIC_INPUTS_COUNT {
        gamma_abc_g1.push(VKey::gamma_abc_g1(i));
    }

    let vk = ark_groth16::VerifyingKey {
        alpha_g1: VKey::alpha_g1(),
        beta_g2: VKey::beta_g2(),
        gamma_g2: VKey::gamma_g2(),
        delta_g2: VKey::delta_g2(),
        gamma_abc_g1,
    };
    ark_groth16::prepare_verifying_key(&vk)
}

async fn setup_validate_nullifier_hashes_test(request_index: usize) -> (ProgramTestContext, Actor, FullSendRequest) {
    let (mut context, mut client) = setup_verification_tests().await;
    let (_, nullifier_0, _) = nullifier_accounts(0, &mut context).await;
    let request = send_requests()[request_index].clone();

    ix_should_succeed(
        ElusivInstruction::init_verification_instruction(
            0,
            [0, 1],
            ProofRequest::Send(request.public_inputs.clone()),
            WritableSignerAccount(client.pubkey),
            &nullifier_0,
            &[],
        ),
        &mut client, &mut context,
    ).await;

    (context, client, request)
}

async fn set_sub_account<T: BorshSerialize>(
    pubkey: &Pubkey,
    t: &T,
    context: &mut ProgramTestContext,
) {
    let mut data = vec![1];
    t.serialize(&mut data).unwrap();
    let lamports = get_balance(pubkey, context).await;
    set_account(context, pubkey, data, lamports).await;
}

#[tokio::test]
async fn test_init_verification_validate_nullifier_hashes_fuzzing() {
    let (mut context, mut client, _) = setup_validate_nullifier_hashes_test(0).await;
    let pending_nullifiers_map_account = pending_nullifiers_map_account(0, &mut context).await;

    let ix = ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
        0,
        [0, 1],
        false,
        &[WritableUserAccount(pending_nullifiers_map_account)],
        &[],
    );

    // Fuzzing
    test_instruction_fuzzing(
        &[],
        ix.clone(),
        &mut client,
        &mut context
    ).await;
}

#[tokio::test]
async fn test_init_verification_validate_nullifier_hashes_duplicates() {
    let (mut context, mut client) = setup_verification_tests().await;
    let (_, nullifier_0, _) = nullifier_accounts(0, &mut context).await;
    let (_, nullifier_1, _) = nullifier_accounts(1, &mut context).await;
    let request = send_requests()[1].clone();

    // Close mt at index 0 
    set_pda_account::<StorageAccount, _>(&mut context, None, |data| {
        let mut storage_account = StorageAccount::new(data, HashMap::new()).unwrap();
        storage_account.set_next_commitment_ptr(&(MT_COMMITMENT_COUNT as u32));
    }).await;
    ix_should_succeed(
        ElusivInstruction::reset_active_merkle_tree_instruction(0, &[], &[]),
        &mut client,
        &mut context
    ).await;

    ix_should_succeed(
        ElusivInstruction::init_verification_instruction(
            0,
            [0, 1],
            ProofRequest::Send(request.public_inputs.clone()),
            WritableSignerAccount(client.pubkey),
            &nullifier_0,
            &nullifier_1,
        ),
        &mut client, &mut context,
    ).await;

    let pending_nullifiers_map_account_0 = pending_nullifiers_map_account(0, &mut context).await;
    let pending_nullifiers_map_account_1 = pending_nullifiers_map_account(1, &mut context).await;
    let nullifier_hash_0 = U256Limbed2::from(request.public_inputs.join_split.nullifier_hashes[0].reduce());
    let nullifier_hash_1 = U256Limbed2::from(request.public_inputs.join_split.nullifier_hashes[1].reduce());

    let ix = ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
        0,
        [0, 1],
        false,
        &[WritableUserAccount(pending_nullifiers_map_account_0)],
        &[WritableUserAccount(pending_nullifiers_map_account_1)],
    );

    // Duplicate nullifier in first MT
    let original_map = pending_nullifiers_map(0, &mut context).await;
    let mut map0 = original_map.clone();
    map0.try_insert(nullifier_hash_0, 0).unwrap();

    set_sub_account(&pending_nullifiers_map_account_0, &map0, &mut context).await;
    ix_should_fail(ix.clone(), &mut client, &mut context).await;
    set_sub_account(&pending_nullifiers_map_account_0, &original_map, &mut context).await;

    // Duplicate in second MT
    let original_map = pending_nullifiers_map(1, &mut context).await;
    let mut map = original_map.clone();
    map.try_insert(nullifier_hash_1, 0).unwrap();

    set_sub_account(&pending_nullifiers_map_account_1, &map, &mut context).await;
    ix_should_fail(ix.clone(), &mut client, &mut context).await;

    // Duplicates in both MTs
    set_sub_account(&pending_nullifiers_map_account_0, &map0, &mut context).await;
    ix_should_fail(ix.clone(), &mut client, &mut context).await;

    // Ignore duplicate nullifier
    let ix = ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
        0,
        [0, 1],
        true,
        &[WritableUserAccount(pending_nullifiers_map_account_0)],
        &[WritableUserAccount(pending_nullifiers_map_account_1)],
    );
    ix_should_succeed(ix.clone(), &mut client, &mut context).await;
    ix_should_fail(ix, &mut client, &mut context).await;

    let map = pending_nullifiers_map(0, &mut context).await;
    assert_eq!(map.len(), 1);
    assert_eq!(*map.get(&nullifier_hash_0).unwrap(), 1);

    let map = pending_nullifiers_map(1, &mut context).await;
    assert_eq!(map.len(), 1);
    assert_eq!(*map.get(&nullifier_hash_1).unwrap(), 1);

    // Second verification with same nullifiers
    ix_should_succeed(
        ElusivInstruction::init_verification_instruction(
            1,
            [0, 1],
            ProofRequest::Send(request.public_inputs.clone()),
            WritableSignerAccount(client.pubkey),
            &nullifier_0,
            &nullifier_1,
        ),
        &mut client, &mut context,
    ).await;

    // Failure due to duplicate
    ix_should_fail(
        ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
            1,
            [0, 1],
            false,
            &[WritableUserAccount(pending_nullifiers_map_account_0)],
            &[WritableUserAccount(pending_nullifiers_map_account_1)],
        ),
        &mut client, &mut context
    ).await;

    let ix = ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
        1,
        [0, 1],
        true,
        &[WritableUserAccount(pending_nullifiers_map_account_0)],
        &[WritableUserAccount(pending_nullifiers_map_account_1)],
    );
    ix_should_succeed(ix.clone(), &mut client, &mut context).await;
    ix_should_fail(ix.clone(), &mut client, &mut context).await;

    let map = pending_nullifiers_map(0, &mut context).await;
    assert_eq!(map.len(), 1);
    assert_eq!(*map.get(&nullifier_hash_0).unwrap(), 2);

    let map = pending_nullifiers_map(1, &mut context).await;
    assert_eq!(map.len(), 1);
    assert_eq!(*map.get(&nullifier_hash_1).unwrap(), 2);
}

#[tokio::test]
async fn test_init_verification_validate_nullifier_hashes_full_map() {
    let (mut context, mut client, _) = setup_validate_nullifier_hashes_test(0).await;
    let pending_nullifiers_map_account = pending_nullifiers_map_account(0, &mut context).await;

    let mut map = pending_nullifiers_map(0, &mut context).await;
    for i in 0..PendingNullifierHashesMap::MAX_ELEMENTS_COUNT - 1 {
        map.try_insert(U256Limbed2([0, i as u128]), 0).unwrap();
    }
    set_sub_account(&pending_nullifiers_map_account, &map, &mut context).await;

    let map = pending_nullifiers_map(0, &mut context).await;
    assert_eq!(map.len(), PendingNullifierHashesMap::MAX_ELEMENTS_COUNT - 1);

    let ix =  ElusivInstruction::init_verification_validate_nullifier_hashes_instruction(
        0,
        [0, 1],
        true,
        &[WritableUserAccount(pending_nullifiers_map_account)],
        &[],
    );

    ix_should_succeed(ix.clone(), &mut client, &mut context).await;
    ix_should_fail(ix.clone(), &mut client, &mut context).await;
}

#[tokio::test]
#[ignore]
async fn test_compute_verification() {
    panic!()
}

#[tokio::test]
#[ignore]
async fn test_finalize_verification() {
    panic!()
}

#[tokio::test]
#[ignore]
async fn test_full_verification() {
    panic!()
}