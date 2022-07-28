pub mod block_chain_tests;
mod general_state_tests;

pub use block_chain_tests::vm;

// pub fn block_chain_tests::

pub fn general_state_tests_debug() {
    general_state_tests::EvmUnitTestDebugger::debug_test();
}

#[cfg(test)]
mod tests {
    use crate::general_state_tests;

    #[test]
    fn general_state_tests() {
        general_state_tests::EvmUnitTestDebugger::debug_test();
    }
}
