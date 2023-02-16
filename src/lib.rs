mod cmn;
use cmn::*;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // contract state
    solution: String,
}

#[near_bindgen]
impl Contract {
    fn hash(s: String) -> String {
        hash(s, env::sha256).encode_hex::<String>()
    }

    // contract methods
    #[init]
    pub fn new(solution: String) -> Self {
        log!("Contract initialized");
        Self { solution }
    }

    pub fn get_solution(&self) -> String {
        self.solution.clone()
    }

    pub fn set_solution(&mut self, solution: String) {
        self.solution = solution;
    }

    pub fn guess_solution(&self, text: String) -> bool {
        if self.solution == Self::hash(text) {
            log!("You guessed the password!");
            true
        } else {
            log!("Wrong password!");
            false
        }
    }
}

/// Unit Test
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn check_guess_solution() {
        run_vm(vm!("dohalee.testnet"));

        let contract = Contract::new(
            "6ac3c336e4094835293a3fed8a4b5fedde1b5e2626d9838fed50693bba00af0e".to_string(),
        );

        let mut logs = logs!["Contract initialized"];

        let guess_result = contract.guess_solution("wrong answer".to_string());
        logs.push("Wrong password!");
        logs.assert();
        assert!(!guess_result, "Expectation: This is incorrect");

        let guess_result = contract.guess_solution("fuck".to_string());
        logs.push("You guessed the password!");
        logs.assert();
        assert!(guess_result, "Expectation: This is correct");
    }
}
