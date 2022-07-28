mod vm;
pub mod vm_test;

pub struct EvmUnitTestDebugger;

impl EvmUnitTestDebugger {
    pub fn debug_test() {
        run_evm_tests::<Self>();
    }
}

impl EvmUnitTest for EvmUnitTestDebugger {
    fn init_state() -> Self {
        Self
    }

    fn try_apply_chain_id(self, _: U256) -> Result<Self, String> {
        Ok(self)
    }

    fn try_apply_network_type(self, _: NetworkType) -> Result<Self, String> {
        Ok(self)
    }

    fn try_apply_environment(self, _: Env) -> Result<Self, String> {
        Ok(self)
    }

    fn try_apply_account(self, _: H160, _: AccountState) -> Result<Self, String> {
        Ok(self)
    }

    fn try_apply_transaction(self, _: Transaction) -> Result<Self, String> {
        Ok(self)
    }

    fn validate_post(&self, _: H256, _: PostTx) -> Result<(), String> {
        Ok(())
    }
}

use ethereum_types::{H160, H256, U256};

use vm_test::{run_evm_tests, AccountState, Env, EvmUnitTest, NetworkType, PostTx, Transaction};
#[test]
fn run_tests() {
    run_evm_tests::<EvmUnitTestDebugger>();
}
