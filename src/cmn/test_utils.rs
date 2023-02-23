#![cfg(all(test, not(target_arch = "wasm32")))]
#![allow(dead_code)]

use core::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
};

pub use near_sdk::{
    //
    test_utils::*,
    testing_env,
    AccountId,
    ParseAccountIdError,
    VMContext,
};

/// Parser for [`AccountId`]
#[inline]
pub fn try_get_account_id(s: impl AsRef<str>) -> Result<AccountId, ParseAccountIdError> {
    s.as_ref().parse()
}

/// Helper function to create a context for testing.
/// It's same as
/// ```
/// # use cmn::test_utils::*;
/// pub fn get_context_builder(predecessor: AccountId) -> VMContextBuilder {
///     let mut builder = VMContextBuilder::new();
///     builder
///         .predecessor_account_id(predecessor.clone())
///         .signer_account_id(predecessor);
///     builder
/// }
/// ```
#[inline]
pub fn get_context_builder(predecessor: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .predecessor_account_id(predecessor.clone())
        .signer_account_id(predecessor);
    builder
}

/// Run VM with given context.
///
/// # Example
///
/// ```
/// # use cmn::test_utils::*;
/// run_vm(vm!("alice.near").is_view(true).build());
/// ```
#[inline]
pub fn run_vm(vm: impl Borrow<VMContextBuilder>) {
    testing_env!(vm.borrow().build());
}

/// A Container for logs that can be asserted against the logs
/// produced by the contract in the VM.
///
/// # Example
/// ```
/// # use cmn::test_utils::*;
/// let mut logs = Logs::new(); // same as logs![];
/// let contract = Contract::new(); // ... log!("Contract initialized");
/// logs.push("Contract initialized");
/// logs.assert();
/// ```
#[derive(Debug, Default)]
pub struct Logs<'a> {
    logs: Vec<&'a str>,
}
impl<'a> Logs<'a> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    /// Assert that the logs match the logs produced by the contract in the VM.
    ///
    /// # Example
    /// ```
    /// # use cmn::test_utils::*;
    /// let mut logs = Logs::new(); // same as logs![];
    /// let contract = Contract::new(); // ... log!("Contract initialized");
    /// logs.push("Contract initialized");
    /// logs.assert();
    /// ```
    #[inline]
    pub fn assert(&self) {
        assert_eq!(self.logs, get_logs());
    }
}
impl<'a> Deref for Logs<'a> {
    type Target = Vec<&'a str>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.logs
    }
}
impl<'a> DerefMut for Logs<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.logs
    }
}
impl<'a> From<Vec<&'a str>> for Logs<'a> {
    #[inline]
    fn from(logs: Vec<&'a str>) -> Self {
        Self { logs }
    }
}

/// Create a VMContextBuilder with given account id as a predecessor.
/// It uses [get_context_builder].
/// # Example
/// ```
/// # use cmn::test_utils::*;
/// vm!("alice.near"); // == get_context_builder(try_get_account_id("alice.near"));
/// ```
#[macro_export]
macro_rules! vm {
        (@GET_ACCOUNT $s:literal) => {
            $crate::test_utils::try_get_account_id($s).expect("Invalid account id")
        };
        (@GET_ACCOUNT $e:expr) => {
            $e
        };
        ($predecessor:expr) => {
            $crate::test_utils::get_context_builder(vm!(@GET_ACCOUNT $predecessor))
        };
    }
pub use vm;

/// Create a container for logs mocks.
///
/// # Example
/// ```
/// # use cmn::test_utils::*;
/// let mut logs = logs!["hello", "world"];
/// logs.push("hello");
/// logs.push("world");
/// logs.assert();
/// ```
#[macro_export]
macro_rules! logs {
        ($($x:expr),*$(,)*) => {
           $crate::test_utils::Logs::from(vec![$($x),*])
        };
    }
pub use logs;
