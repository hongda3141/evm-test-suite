pub mod vm;

// use ethers_core::utils::hex;   // no need for this crate. use hex crate
// use protocol::types::{Bytes, LegacyTransaction, TransactionAction};
// use protocol::types::{
//     MemoryAccount, SignatureComponents, SignedTransaction, UnsignedTransaction,
//     UnverifiedTransaction,
// };
use ethereum_types::{H160, H256, U256};
use evm::backend::MemoryAccount;
// use protocol::types::{H160, H256, U256};
use serde::{Deserialize, Deserializer};
use std::{collections::BTreeMap, io::BufReader, mem::size_of, str::FromStr};

fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    H256::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    H160::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_hex_data<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    hex::decode(
        s[0..2]
            .eq("0x")
            .then(|| s[2..].to_string())
            .ok_or("Missing '0x' prefix for hex data")
            .map_err(serde::de::Error::custom)?,
    )
    .map_err(serde::de::Error::custom)
}

fn deserialize_account_storage<'de, D>(deserializer: D) -> Result<BTreeMap<H256, H256>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = <BTreeMap<String, String>>::deserialize(deserializer)?;
    let feel_zeros = |mut val: String| -> Result<String, String> {
        val = val[0..2]
            .eq("0x")
            .then(|| val[2..].to_string())
            .ok_or("Missing '0x' prefix for hex data")?;

        while val.len() < size_of::<H256>() * 2 {
            val = "00".to_string() + &val;
        }
        val = "0x".to_string() + &val;
        Ok(val)
    };
    Ok(map
        .into_iter()
        .map(|(k, v)| {
            (
                H256::from_str(&feel_zeros(k).unwrap()).expect("Can not parse account storage key"),
                H256::from_str(&feel_zeros(v).unwrap()).expect("Can not parse account storage key"),
            )
        })
        .collect())
}

fn deserialize_accounts<'de, D>(deserializer: D) -> Result<BTreeMap<H160, AccountState>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = <BTreeMap<String, AccountState>>::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (H160::from_str(&k).unwrap(), v))
        .collect())
}
#[derive(Debug, Clone, Copy)]
pub enum NetworkType {
    Istanbul,
    Berlin,
    London,
    Merge,
}

impl<'de> Deserialize<'de> for NetworkType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Istanbul" => Ok(Self::Istanbul),
            "Berlin" => Ok(Self::Berlin),
            "London" => Ok(Self::London),
            "Merge" => Ok(Self::Merge),
            network => Err(format!("Not known network type, {}", network)),
        }
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Debug)]
pub struct AccountState {
    #[serde(deserialize_with = "deserialize_u256")]
    pub balance: U256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub code: Vec<u8>,
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce: U256,
    #[serde(deserialize_with = "deserialize_account_storage")]
    pub storage: BTreeMap<H256, H256>,
}

impl TryInto<MemoryAccount> for AccountState {
    type Error = ();

    fn try_into(self) -> Result<MemoryAccount, Self::Error> {
        Ok(MemoryAccount {
            balance: self.balance,
            code: self.code,
            nonce: self.nonce,
            storage: self.storage,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallTransaction {
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub data: Vec<u8>,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_price: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub nonce: U256,
    #[serde(deserialize_with = "deserialize_h160")]
    pub sender: H160,
    #[serde(deserialize_with = "deserialize_h160")]
    pub to: H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub value: U256,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub r: Vec<u8>,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub s: Vec<u8>,
    #[serde(deserialize_with = "deserialize_hex_data")]
    pub v: Vec<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    #[serde(deserialize_with = "deserialize_h160")]
    pub coinbase: H160,
    #[serde(deserialize_with = "deserialize_u256")]
    pub difficulty: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub gas_limit: U256,
    #[serde(deserialize_with = "deserialize_h256")]
    pub hash: H256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub number: U256,
    #[serde(deserialize_with = "deserialize_u256")]
    pub timestamp: U256,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    block_header: BlockHeader,
    transactions: Vec<CallTransaction>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    #[serde(deserialize_with = "deserialize_accounts")]
    pre: BTreeMap<H160, AccountState>,
    network: NetworkType,
    genesis_block_header: BlockHeader,
    blocks: Vec<Block>,
    #[serde(deserialize_with = "deserialize_accounts")]
    post_state: BTreeMap<H160, AccountState>,
}

#[derive(Debug)]
pub struct TestNum {
    total: i32,
    failed: i32,
    skipped: i32,
}

pub trait TestEvmState: Sized {
    fn init_state() -> Self;

    fn try_apply_network_type(self, net_type: NetworkType) -> Result<Self, String>;

    fn try_apply_accounts<I>(self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = (H160, AccountState)>;

    fn try_apply_block_header(self, block_header: &BlockHeader) -> Result<Self, String>;

    fn try_apply_transaction(self, tx: CallTransaction) -> Result<Self, String>;

    fn validate_account(
        &self,
        address: H160,
        coinbase: H160,
        skip_coinbase: bool,
        account: AccountState,
    ) -> Result<(), String>;

    fn try_apply_block(mut self, block: Block) -> Result<Self, String> {
        self = self.try_apply_block_header(&block.block_header)?;
        for transaction in block.transactions {
            self = self.try_apply_transaction(transaction)?;
        }

        Ok(self)
    }

    fn try_apply_blocks<I>(mut self, iter: I) -> Result<Self, String>
    where
        I: Iterator<Item = Block>,
    {
        for block in iter {
            self = self.try_apply_block(block)?;
        }
        Ok(self)
    }

    fn validate_accounts<I>(&self, iter: I, coinbase: H160, skip_coinbase: bool) -> TestNum
    where
        I: Iterator<Item = (H160, AccountState)>,
    {
        let mut sum: TestNum = TestNum {
            total: 0,
            failed: 0,
            skipped: 0,
        };
        for (address, account) in iter {
            // self.validate_account(address, coinbase, skip_coinbase, account)
            //     .unwrap_or_else(|err| {
            //         if !err.contains("skip") {
            //             println!("{}", err);
            //             sum.failed += 1
            //         } else {
            //             sum.skipped += 1;
            //         }
            //     });
            self.validate_account(address, coinbase, skip_coinbase, account)
                .unwrap_or_else(|err| {
                    if !err.contains("skip") {
                        println!("{}", err);
                        sum.failed += 1;
                        // panic!("{:#?}", err);
                        panic!();
                    } else {
                        sum.skipped += 1;
                    }
                });
            sum.total += 1;
        }
        sum
    }
}

fn run_evm_test<State: TestEvmState>(
    test: &str,
    single_case: &str,
    skip_coinbase: bool,
) -> TestNum {
    let mut sum: TestNum = TestNum {
        total: 0,
        failed: 0,
        skipped: 0,
    };
    let reader = BufReader::new(test.as_bytes());

    let test: BTreeMap<String, TestCase> =
        serde_json::from_reader(reader).expect("Parse test cases failed");

    for (test_name, test_case) in test {
        if single_case != test_name && !single_case.is_empty() {
            continue;
        }
        println!("\nRunning test: {} ...", test_name);
        // test case in this list get length of transaction.r/s is 31 bytes which should
        // be 32 bytes.
        let black_list = vec![
            "calldatasize_d4g0v0_Istanbul",
            "calldatasize_d4g0v0_Berlin",
            "calldatasize_d4g0v0_London",
            "calldatasize_d4g0v0_Merge",
            "dup_d8g0v0_Berlin",
            "dup_d8g0v0_Istanbul",
            "dup_d8g0v0_London",
            "dup_d8g0v0_Merge",
            "push_d8g0v0_Berlin",
            "push_d8g0v0_Istanbul",
            "push_d8g0v0_London",
            "push_d8g0v0_Merge",
            "sha3_d8g0v0_Berlin",
            "sha3_d8g0v0_Istanbul",
            "sha3_d8g0v0_London",
            "sha3_d8g0v0_Merge",
            "swap_d8g0v0_Berlin",
            "swap_d8g0v0_Istanbul",
            "swap_d8g0v0_London",
            "swap_d8g0v0_Merge",
        ];
        if black_list.contains(&&*test_name) {
            continue;
        }
        let coinbase = test_case.genesis_block_header.coinbase;
        let state = State::init_state()
            .try_apply_network_type(test_case.network)
            .unwrap()
            .try_apply_accounts(test_case.pre.into_iter())
            .unwrap()
            .try_apply_block_header(&test_case.blocks[0].block_header)
            .unwrap()
            .try_apply_blocks(test_case.blocks.into_iter())
            .unwrap();
        let num =
            state.validate_accounts(test_case.post_state.into_iter(), coinbase, skip_coinbase);

        sum.failed += num.failed;
        sum.total += num.total;
        sum.skipped += num.skipped;
    }
    sum
}

fn print_result(num: TestNum) {
    println!(
        "*************************************************************************************************************"
    );
    println!(
        "evm compatibility test result: total {} test cases; skipped {} cases; failed {} cases; success {} cases.",
        num.total,
        num.skipped,
        num.failed,
        num.total - num.failed - num.skipped,
    );
    println!(
        "*************************************************************************************************************"
    );
}

pub fn run_tests<State: TestEvmState>(skip_coinbase: bool) {
    let tests = vec![
        vm::BLOCK_INFO,
        vm::CALL_DATA_COPY,
        vm::CALL_DATA_LOAD,
        vm::CALL_DATA_SIZE,
        vm::DUP,
        vm::ENV_INFO,
        vm::PUSH,
        vm::RANDOM,
        vm::SHA3,
        vm::SUICIDE,
        vm::SWAP,
    ];
    let mut num: TestNum = TestNum {
        total: 0,
        failed: 0,
        skipped: 0,
    };
    for test in tests {
        let sum = run_evm_test::<State>(test, String::new().as_str(), skip_coinbase);
        num.total += sum.total;
        num.failed += sum.failed;
    }
    print_result(num);
}

pub fn run_single_test<State: TestEvmState>(test: &str, single_case: &str, skip_coinbase: bool) {
    let num = run_evm_test::<State>(test, single_case, skip_coinbase);
    print_result(num);
}

#[cfg(test)]
mod test {

    use super::*;

    struct EvmState;

    impl TestEvmState for EvmState {
        fn init_state() -> Self {
            Self
        }

        fn validate_account(
            &self,
            _: H160,
            _: H160,
            _: bool,
            _: AccountState,
        ) -> Result<(), String> {
            // Err(String::from("11"))
            Ok(())
        }

        fn try_apply_network_type(self, _: NetworkType) -> Result<Self, String> {
            Ok(self)
        }

        fn try_apply_accounts<I>(self, _: I) -> Result<Self, String>
        where
            I: Iterator<Item = (H160, AccountState)>,
        {
            Ok(self)
        }

        fn try_apply_block_header(self, _: &BlockHeader) -> Result<Self, String> {
            Ok(self)
        }

        fn try_apply_transaction(self, _: CallTransaction) -> Result<Self, String> {
            Ok(self)
        }
    }

    #[test]
    fn run_examples() {
        run_tests::<EvmState>(true);
    }

    #[test]
    fn run_example() {
        run_single_test::<EvmState>(vm::BLOCK_INFO, "", true);
    }
}
