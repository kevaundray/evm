pub(crate) mod modexp_contract {
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
    pub(crate) const MODEXP_ADDRESS: Address = address!("0x0000000000000000000000000000000000000005");

    /// EVM bytecode deployed at [`MODEXP_ADDRESS`] at Osaka.
    ///
    /// Placeholder — will be replaced with the real modexp contract bytecode once finalized.
    pub(crate) static MODEXP_CONTRACT_CODE: Bytes = Bytes::from_static(&[0x00]);
}

use crate::{block::BlockExecutionError, Evm};
use modexp_contract::{MODEXP_ADDRESS, MODEXP_CONTRACT_CODE};
use alloy_hardforks::EthereumHardforks;
use alloy_primitives::{keccak256, U256};
use revm::{
    context::Block,
    state::{Account, AccountInfo, Bytecode, EvmState},
    Database, DatabaseCommit,
};

/// Deploys the modexp contract bytecode at address `0x05` on the first Osaka block.
///
/// This is a state injection — it directly inserts an account with
/// the modexp contract code. It is idempotent: if the account already exists and is
/// non-empty, no changes are made.
pub(crate) fn deploy_modexp_contract(
    spec: &impl EthereumHardforks,
    evm: &mut impl Evm<DB: Database + DatabaseCommit>,
) -> Result<(), BlockExecutionError> {
    if !spec.is_osaka_active_at_timestamp(evm.block().timestamp().saturating_to()) {
        return Ok(());
    }

    let needs_deploy = evm
        .db_mut()
        .basic(MODEXP_ADDRESS)
        .map_err(|e| {
            BlockExecutionError::msg(alloc::format!("failed to read modexp account: {e}"))
        })?
        .is_none_or(|info| info.is_empty());

    if needs_deploy {
        let code = MODEXP_CONTRACT_CODE.clone();
        let code_hash = keccak256(&code);
        let info = AccountInfo {
            nonce: 1,
            balance: U256::ZERO,
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
