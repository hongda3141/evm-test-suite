# evm-test-suite
ethereum standard vm compatibility test suite. 
tests cases source: https://github.com/ethereum/tests


## EVM Testing
Download the required test files: 
```shell
$ git clone git@github.com:hongda3141/evm-test-suite.git
$ cd evm-test-suite
$ git submodule update --init --recursive  --depth=1
```
You can run tests with the following commands:
```shell
$ cargo test --package evm-test-suite --lib -- block_chain_tests::test::run_example --exact --nocapture
```
