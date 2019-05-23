use super::*;
use std::fmt::Debug;

mod bls_aggregate_pubkeys;
mod bls_aggregate_sigs;
mod bls_g2_compressed;
mod bls_g2_uncompressed;
mod bls_priv_to_pub;
mod bls_sign_msg;
mod operations_deposit;
mod operations_exit;
mod operations_transfer;
mod ssz_generic;
mod ssz_static;

pub use bls_aggregate_pubkeys::*;
pub use bls_aggregate_sigs::*;
pub use bls_g2_compressed::*;
pub use bls_g2_uncompressed::*;
pub use bls_priv_to_pub::*;
pub use bls_sign_msg::*;
pub use operations_deposit::*;
pub use operations_exit::*;
pub use operations_transfer::*;
pub use ssz_generic::*;
pub use ssz_static::*;

pub trait Case: Debug {
    /// An optional field for implementing a custom description.
    ///
    /// Defaults to "no description".
    fn description(&self) -> String {
        "no description".to_string()
    }

    /// Execute a test and return the result.
    ///
    /// `case_index` reports the index of the case in the set of test cases. It is not strictly
    /// necessary, but it's useful when troubleshooting specific failing tests.
    fn result(&self, case_index: usize) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct Cases<T> {
    pub test_cases: Vec<T>,
}

impl<T> EfTest for Cases<T>
where
    T: Case + Debug,
{
    fn test_results(&self) -> Vec<CaseResult> {
        self.test_cases
            .iter()
            .enumerate()
            .map(|(i, tc)| CaseResult::new(i, tc, tc.result(i)))
            .collect()
    }
}

impl<T: YamlDecode> YamlDecode for Cases<T> {
    /// Decodes a YAML list of test cases
    fn yaml_decode(yaml: &String) -> Result<Self, Error> {
        let mut p = 0;
        let mut elems: Vec<&str> = yaml
            .match_indices("\n- ")
            // Skip the `\n` used for matching a new line
            .map(|(i, _)| i + 1)
            .map(|i| {
                let yaml_element = &yaml[p..i];
                p = i;

                yaml_element
            })
            .collect();

        elems.push(&yaml[p..]);

        let test_cases = elems
            .iter()
            .map(|s| {
                // Remove the `- ` prefix.
                let s = &s[2..];
                // Remove a single level of indenting.
                s.replace("\n  ", "\n")
            })
            .map(|s| T::yaml_decode(&s.to_string()).unwrap())
            .collect();

        Ok(Self { test_cases })
    }
}
