/// Gas reports
pub mod gas_report;

/// The Forge test runner
mod runner;
pub use runner::{ContractRunner, SuiteResult, TestKind, TestKindGas, TestResult};

/// Forge test runners for multiple contracts
mod multi_runner;
pub use multi_runner::{MultiContractRunner, MultiContractRunnerBuilder};

pub trait TestFilter {
    fn matches_test(&self, test_name: impl AsRef<str>) -> bool;
    fn matches_contract(&self, contract_name: impl AsRef<str>) -> bool;
    fn matches_path(&self, path: impl AsRef<str>) -> bool;
}

/// The Forge EVM backend
pub use foundry_evm::*;

#[cfg(test)]
pub mod test_helpers {
    use crate::TestFilter;
    use ethers::{
        prelude::{Lazy, ProjectCompileOutput},
        solc::{Project, ProjectPathsConfig},
        types::{Address, U256},
    };
    use foundry_evm::{
        executor::{
            builder::Backend,
            opts::{Env, EvmOpts},
            DatabaseRef, Executor, ExecutorBuilder,
        },
        fuzz::FuzzedExecutor,
        CALLER,
    };
    use std::str::FromStr;

    pub static PROJECT: Lazy<Project> = Lazy::new(|| {
        let paths = ProjectPathsConfig::builder()
            .root("../testdata")
            .sources("../testdata")
            .build()
            .unwrap();
        Project::builder().paths(paths).ephemeral().no_artifacts().build().unwrap()
    });

    pub static COMPILED: Lazy<ProjectCompileOutput> = Lazy::new(|| (*PROJECT).compile().unwrap());

    pub static EVM_OPTS: Lazy<EvmOpts> = Lazy::new(|| EvmOpts {
        env: Env {
            gas_limit: 18446744073709551615,
            chain_id: Some(99),
            tx_origin: Address::from_str("00a329c0648769a73afac7f9381e08fb43dbea72").unwrap(),
            ..Default::default()
        },
        sender: Address::from_str("00a329c0648769a73afac7f9381e08fb43dbea72").unwrap(),
        initial_balance: U256::MAX,
        ffi: true,
        ..Default::default()
    });

    pub fn test_executor() -> Executor<Backend> {
        ExecutorBuilder::new()
            .with_cheatcodes(false)
            .with_config((*EVM_OPTS).evm_env())
            .build(Backend::simple())
    }

    pub fn fuzz_executor<DB: DatabaseRef>(executor: &Executor<DB>) -> FuzzedExecutor<DB> {
        let cfg = proptest::test_runner::Config { failure_persistence: None, ..Default::default() };

        FuzzedExecutor::new(executor, proptest::test_runner::TestRunner::new(cfg), *CALLER)
    }

    pub mod filter {
        use super::*;
        use regex::Regex;

        pub struct Filter {
            test_regex: Regex,
            contract_regex: Regex,
            path_regex: Regex,
        }

        impl Filter {
            pub fn new(test_pattern: &str, contract_pattern: &str, path_pattern: &str) -> Self {
                Filter {
                    test_regex: Regex::new(test_pattern).unwrap(),
                    contract_regex: Regex::new(contract_pattern).unwrap(),
                    path_regex: Regex::new(path_pattern).unwrap(),
                }
            }

            pub fn matches_all() -> Self {
                Filter {
                    test_regex: Regex::new(".*").unwrap(),
                    contract_regex: Regex::new(".*").unwrap(),
                    path_regex: Regex::new(".*").unwrap(),
                }
            }
        }

        impl TestFilter for Filter {
            fn matches_test(&self, test_name: impl AsRef<str>) -> bool {
                self.test_regex.is_match(test_name.as_ref())
            }

            fn matches_contract(&self, contract_name: impl AsRef<str>) -> bool {
                self.contract_regex.is_match(contract_name.as_ref())
            }

            fn matches_path(&self, path: impl AsRef<str>) -> bool {
                self.path_regex.is_match(path.as_ref())
            }
        }
    }
}
