#![allow(dead_code)]

#![feature(proc_macro)]
#![feature(alloc)]

// Conract doesn't need standard library and the `main` function.
// `cargo test` requires `std` and it provided the `main` which is why the "std" feature should be turned on for `cargo test`
#![cfg_attr(not(feature="std"), no_main)]
#![cfg_attr(not(feature="std"), no_std)]

extern crate alloc;
extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;

use pwasm_abi::eth::EndpointInterface;
use pwasm_std::{storage, ext};
use pwasm_std::hash::{Address, H256};

struct Entry {
    key: H256,
    value: [u8; 32]
}

struct Storage {
    table: Vec<Entry>,
}

/// TODO: optimise by impl hashtable
impl Storage {
    fn with_capacity(cap: usize) -> Storage {
        Storage {
            table: Vec::with_capacity(cap)
        }
    }

    fn read(&mut self, key: &H256) -> [u8; 32] {
        // First: lookup in the table
        for entry in &self.table {
            if *key == entry.key {
                return entry.value.clone();
            }
        }
        // Second: read from the storage
        let value = storage::read(key);
        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
        value
    }

    fn write(&mut self, key: &H256, value: &[u8; 32]) {
        storage::write(key, value);
        for entry in &mut self.table {
            if *key == entry.key {
                entry.value = *value;
                return;
            }
        }
        self.table.push(Entry {
            key: key.clone(),
            value: value.clone()
        });
    }
}


pub mod contract {
    #![allow(non_snake_case)]

    extern crate pwasm_token_contract;
    use alloc::vec::Vec;

    use pwasm_std::{storage, ext};
    use pwasm_std::hash::{Address, H256};
    use pwasm_std::bigint::U256;

    use pwasm_abi_derive::eth_abi;

    use contract::pwasm_token_contract::TokenContract;
    use self::pwasm_token_contract::Client as Token;

    #[eth_abi(Endpoint, Client)]
    pub trait RepoContract {
        fn constructor(&mut self,
            borrower: Address,
            lender: Address,
            loan_token: Address,
            security_token: Address,
            loan_amount: U256,
            security_amount: U256,
            interest_rate: U256,
            activation_deadline: u64,
            return_deadline: u64);

        fn accept(&mut self) -> bool;

        fn terminate(&mut self) -> bool;

        #[event]
        fn LendAccepted(&mut self);
        #[event]
        fn BorrowAccepted(&mut self);

    }

    static BORROWER_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LENDER_KEY: H256 = H256([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LOAN_TOKEN_KEY: H256 = H256([4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static SECURITY_TOKEN_KEY: H256 = H256([5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LOAN_AMOUNT_KEY: H256 = H256([6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static SECURITY_AMOUNT_KEY: H256 = H256([7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static INTEREST_RATE_KEY: H256 = H256([8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static ACTIVATION_DEADLINE_KEY: H256 = H256([9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static RETURN_DEADLINE_KEY: H256 = H256([10,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static BORROW_ACCEPTED_KEY: H256 = H256([11,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LEND_ACCEPTED_KEY: H256 = H256([12,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

    static DIVISOR: U256 = U256([100,0,0,0]);

    pub struct RepoContractInstance {
        storage: super::Storage
    }

    impl RepoContractInstance {
        pub fn new() -> RepoContractInstance {
            RepoContractInstance {
                storage: super::Storage::with_capacity(10)
            }
        }
        pub fn read_borrower_address(&mut self) -> Address {
            H256::from(self.storage.read(&BORROWER_KEY)).into()
        }

        pub fn read_lender_address(&mut self) -> Address {
            H256::from(self.storage.read(&LENDER_KEY)).into()
        }

        pub fn read_loan_token_address(&mut self) -> Address {
            H256::from(self.storage.read(&LOAN_TOKEN_KEY)).into()
        }

        pub fn read_security_token_address(&mut self) -> Address {
            H256::from(self.storage.read(&SECURITY_TOKEN_KEY)).into()
        }

        pub fn read_loan_amount(&mut self) -> U256 {
            self.storage.read(&LOAN_AMOUNT_KEY).into()
        }

        pub fn read_security_amount(&mut self) -> U256 {
            self.storage.read(&SECURITY_AMOUNT_KEY).into()
        }

        pub fn read_interest_rate(&mut self) -> U256 {
            self.storage.read(&INTEREST_RATE_KEY).into()
        }

        // Activation deadline timestamp
        pub fn read_activation_deadline(&mut self) -> u64 {
            U256::from(self.storage.read(&ACTIVATION_DEADLINE_KEY)).into()
        }

        // Return deadline timestamp
        pub fn read_return_deadline(&mut self) -> u64 {
            U256::from(self.storage.read(&RETURN_DEADLINE_KEY)).into()
        }

        pub fn read_borrower_acceptance(&mut self) -> bool {
            let value = U256::from(self.storage.read(&BORROW_ACCEPTED_KEY));
            if value == 0.into() {
                false
            } else {
                true
            }
        }

        pub fn read_lender_acceptance(&mut self) -> bool {
            let value = U256::from(self.storage.read(&LEND_ACCEPTED_KEY));
            if value == 0.into() {
                false
            } else {
                true
            }
        }
    }

    impl RepoContract for RepoContractInstance {
        /// A contract constructor implementation.
        fn constructor(&mut self,
            borrower: Address,
            lender: Address,
            loan_token: Address,
            security_token: Address,
            loan_amount: U256,
            security_amount: U256,
            interest_rate: U256,
            activation_deadline: u64,
            return_deadline: u64) {

            self.storage.write(&BORROWER_KEY, &H256::from(borrower).into());
            self.storage.write(&LENDER_KEY, &H256::from(lender).into());
            self.storage.write(&LOAN_TOKEN_KEY, &H256::from(loan_token).into());
            self.storage.write(&SECURITY_TOKEN_KEY, &H256::from(security_token).into());
            self.storage.write(&LOAN_AMOUNT_KEY, &loan_amount.into());
            self.storage.write(&SECURITY_AMOUNT_KEY, &security_amount.into());
            self.storage.write(&INTEREST_RATE_KEY, &interest_rate.into());
            self.storage.write(&ACTIVATION_DEADLINE_KEY, &U256::from(activation_deadline).into());
            self.storage.write(&RETURN_DEADLINE_KEY, &U256::from(return_deadline).into());
        }

        // Tries to activate contract
        fn accept(&mut self) -> bool {
            let sender = ext::sender();
            if ext::timestamp() > self.read_activation_deadline() {
                ext::suicide(&sender);
            }

            let lender_address = self.read_lender_address();
            let borrower_address = self.read_borrower_address();

            // Accept by borrower
            if sender == borrower_address {
                self.storage.write(&BORROW_ACCEPTED_KEY, &U256::from(1).into());
            }
            // Accept by lender
            else if sender == lender_address {
                self.storage.write(&LEND_ACCEPTED_KEY, &U256::from(1).into());
            } else {
                panic!("Only for participants");
            }

            // Wait for all parties to accept
            if !(self.read_borrower_acceptance() && self.read_lender_acceptance()) {
                return false;
            }

            let mut loan_token = Token::new(self.read_loan_token_address());
            let mut security_token = Token::new(self.read_security_token_address());
            let loan_amount = self.read_loan_amount();
            let security_amount = self.read_security_amount();

            let this_contract_address = ext::address();
            // Transfer security from borrower_address to the contract address
            assert!(security_token.transferFrom(borrower_address, this_contract_address, security_amount));
            // Transfer loan to the borrower address
            assert!(loan_token.transferFrom(lender_address, borrower_address, loan_amount));

            return true;
        }

        fn terminate(&mut self) -> bool {
            let lender_address = self.read_lender_address();
            let borrower_address = self.read_borrower_address();
            let mut loan_token = Token::new(self.read_loan_token_address());
            let mut security_token = Token::new(self.read_security_token_address());
            let sender = ext::sender();
            let loan_amount = self.read_loan_amount();
            let security_amount = self.read_security_amount();
            let interest_amount = (loan_amount / DIVISOR) * self.read_interest_rate();
            let return_amount = loan_amount + interest_amount;

            if ext::timestamp() <= self.read_return_deadline() {
                if sender != borrower_address {
                    return false;
                }
                assert!(loan_token.transferFrom(borrower_address, lender_address, return_amount));
                assert!(security_token.transfer(borrower_address, security_amount));
                ext::suicide(&sender);
            } else {
                assert!(security_token.transfer(lender_address, security_amount));
                ext::suicide(&sender);
            }
        }
    }
}

/// The main function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = contract::Endpoint::new(contract::RepoContractInstance::new());
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = contract::Endpoint::new(contract::RepoContractInstance::new());
    endpoint.dispatch_ctor(&args);
}

#[cfg(feature="std")]
#[macro_use]
extern crate pwasm_test;

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    extern crate std;
    use pwasm_test;
    use super::contract::*;
    use self::pwasm_test::{External, ExternalBuilder, ExternalInstance, get_external, set_external};
    use pwasm_std::bigint::U256;
    use pwasm_std::hash::{Address};

    test_with_external!(
        ExternalBuilder::new().build(),
        should_create_contract_with_storage {
            let mut contract = RepoContractInstance::new();
            let borrower = Address::from("0xea674fdde714fd979de3edf0f56aa9716b898ec8");
            let lender =  Address::from("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1");
            let loan_token = Address::from("0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
            let security_token = Address::from("0xcd1722f2947def4cf144679da39c4c32bdc35681");
            let loan_amount: U256 = 10000.into();
            let security_amount: U256 = 50000.into();
            let interest_rate: U256 = 3.into();
            let activation_deadline: u64 = 10;
            let return_deadline: u64 = 20;

            contract.constructor(borrower.clone(), lender.clone(), loan_token.clone(), security_token.clone(),
                loan_amount, security_amount, interest_rate, activation_deadline, return_deadline);
            assert_eq!(contract.read_borrower_address(), borrower);
            assert_eq!(contract.read_lender_address(), lender);
            assert_eq!(contract.read_loan_token_address(), loan_token);
            assert_eq!(contract.read_security_token_address(), security_token);
            assert_eq!(contract.read_loan_amount(), loan_amount);
            assert_eq!(contract.read_security_amount(), security_amount);
            assert_eq!(contract.read_interest_rate(), interest_rate);
            assert_eq!(contract.read_activation_deadline(), activation_deadline);
            assert_eq!(contract.read_return_deadline(), return_deadline);
        }
    );

    test_with_external!(
        ExternalBuilder::new()
            .sender("0xea674fdde714fd979de3edf0f56aa9716b898ec8".into())
            .timestamp(5)
            .build(),
        should_pledge {
            let mut contract = RepoContractInstance::new();
            let borrower = Address::from("0xea674fdde714fd979de3edf0f56aa9716b898ec8");
            let lender =  Address::from("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1");
            let loan_token = Address::from("0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
            let security_token = Address::from("0xcd1722f2947def4cf144679da39c4c32bdc35681");
            let loan_amount: U256 = 10000.into();
            let security_amount: U256 = 50000.into();
            let interest_rate: U256 = 3.into();
            let activation_deadline: u64 = 10;
            let return_deadline: u64 = 20;

            contract.constructor(borrower,
             lender.clone(), loan_token.clone(), security_token.clone(),
                loan_amount, security_amount, interest_rate, activation_deadline, return_deadline);

            assert_eq!(contract.accept(), false);

            let spenderExternal = ExternalBuilder::from(get_external::<ExternalInstance>())
                .sender(lender)
                .build();
            set_external(Box::new(spenderExternal));
            assert_eq!(contract.accept(), true);
            // assert_eq!(contract.read_borrower_acceptance(), true)
        }
    );

}
