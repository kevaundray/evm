/// Constants for the modexp contract deployment at Osaka.
pub mod modexp_contract {
    // TODO: This would go into alloy-eips -- leaving it here so we don't need
    // to modify that repo for now.

    //! [EIP-XXXX] constants for the modexp contract deployment at H*.
    //!
    //! At the H* hardfork, the modexp precompile at address `0x05` is replaced with deployed
    //! EVM bytecode. This module provides the address and bytecode constants.
    //!
    //! [EIP-XXXX]: https://eips.ethereum.org/EIPS/eip-XXXX

    use alloy_primitives::{address, Address, Bytes};

    /// Address of the modexp precompile / deployed contract.
    pub const MODEXP_ADDRESS: Address = address!("0x0000000000000000000000000000000000000005");

    /// EVM bytecode deployed at [`MODEXP_ADDRESS`] at Osaka.
    ///
    /// Placeholder — will be replaced with the real modexp contract bytecode once finalized.
    pub static MODEXP_CONTRACT_CODE: Bytes = Bytes::from_static(&[0x00]);
}

use crate::{block::BlockExecutionError, Evm};
use alloy_hardforks::EthereumHardforks;
use alloy_primitives::keccak256;
use modexp_contract::{MODEXP_ADDRESS, MODEXP_CONTRACT_CODE};
use revm::{
    context::Block,
    state::{Account, AccountInfo, Bytecode, EvmState},
    Database, DatabaseCommit,
};

/// Deploys the modexp contract bytecode at address `0x05` on the first Osaka block.
///
/// This is a state injection — it directly inserts an account with
/// the modexp contract code, preserving any existing balance. It is idempotent:
/// if the account already has code, no changes are made.
pub(crate) fn deploy_modexp_contract(
    spec: &impl EthereumHardforks,
    evm: &mut impl Evm<DB: Database + DatabaseCommit>,
) -> Result<(), BlockExecutionError> {
    if !spec.is_osaka_active_at_timestamp(evm.block().timestamp().saturating_to()) {
        return Ok(());
    }

    // Note. If we had access to the parent timestamp, we could avoid the db lookup and do:
    /*
    if !spec.is_osaka_active_at_timestamp(current_timestamp) {
        return Ok(());
    }
    if spec.is_osaka_active_at_timestamp(parent_timestamp) {
        return Ok(());
    }
    */
    // This would only ever trigger on the first Osaka block

    let existing = evm
        .db_mut()
        .basic(MODEXP_ADDRESS)
        .map_err(|e| {
            BlockExecutionError::msg(alloc::format!("failed to read modexp account: {e}"))
        })?
        .unwrap_or_default();

    // Only check for non-empty code, since folks may have sent
    // funds to the precompile address
    if existing.is_empty_code_hash() || existing.code_hash.is_zero() {
        let code = MODEXP_CONTRACT_CODE.clone();
        let code_hash = keccak256(&code);
        let info = AccountInfo {
            nonce: 1, // Convention is to set nonce to 1. It won't get cleared up via EIP161 since it has code.
            balance: existing.balance,
            code_hash,
            code: Some(Bytecode::new_raw(code)),
            ..Default::default()
        };
        let account = Account::from(info).with_touched_mark().with_created_mark();

        let mut state = EvmState::default();
        state.insert(MODEXP_ADDRESS, account);
        evm.db_mut().commit(state);
    }

    Ok(())
}
