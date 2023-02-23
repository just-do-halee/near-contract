#![allow(dead_code)]

/// Helper functions for hashing
///
/// # Example
/// ```
/// # use cmn::*;
/// let hashed = hash("hello world", env::sha256);
/// let hex = hashed.encode_hex::<String>();
/// ```
#[inline]
pub fn hash<I: AsRef<[u8]>, O: AsRef<[u8]>>(s: I, h: fn(&[u8]) -> O) -> O {
    h(s.as_ref())
}

/// Assert when the contract has been initialized.
///
/// # Example
/// ```
/// # use cmn::*;
/// // assert_init!([panic message; default: "Already initialized"]);
/// assert_init!();
/// // == assert_init!("Already initialized");
/// ```
#[macro_export]
macro_rules! assert_init {
    ($message:expr) => {
        assert!(!env::state_exists(), $message);
    };
    () => {
        assert_init!("Already initialized");
    };
}
pub use assert_init;

/// Assert when the contract has not been initialized. [require!] version.
///
/// # Example
/// ```
/// # use cmn::*;
/// // require_init!([panic message; default: "Already initialized"]);
/// require_init!();
/// // == require_init!("Already initialized");
/// ```
#[macro_export]
macro_rules! require_init {
    ($message:expr) => {
        require!(!env::state_exists(), $message);
    };
    () => {
        require_init!("Already initialized");
    };
}
pub use require_init;
