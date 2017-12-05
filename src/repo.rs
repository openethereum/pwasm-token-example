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

pub mod contract {
    #![allow(non_snake_case)]
    use alloc::vec::Vec;

    use pwasm_std::{storage, ext};
    use pwasm_std::hash::{Address, H256};
    use pwasm_std::bigint::U256;

    use pwasm_abi_derive::eth_abi;

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

        // TODO: update cache
        fn write(&mut self, key: &H256, value: &[u8; 32]) {
            storage::write(key, value);
        }
    }

    #[eth_abi(Endpoint, Client)]
    pub trait RepoContract {
        fn constructor(&mut self,
            borrower: Address,
            lender: Address,
            borrowed_token: Address,
            security_token: Address,
            amount_to_borrow: U256,
            security_amount: U256,
            interest_rate: U256,
            activation_deadline: u64,
            return_deadline: u64);


        fn lend(&mut self) -> bool;

        /// If `borrower` is `ext::sender` and `activation_deadline` < `ext::timestamp` and pledge isn't made yet
        /// Transfers `security_amount` of `security_token` to the current contract address
        fn pledge(&mut self) -> bool;

        /// Transfer `security_amount` of `security_token` to `lender` address and `ext::suicide()`
        /// if `ext::sender` is `lender` and `return_deadline` is > `ext::timestamp()`
        fn redeem(&mut self) -> bool;

        /// Transfer `security_amount` of `security_token` from contract address to `lender` address and `ext::suicide()`
        /// if `ext::sender` is `lender` and `return_deadline` is > `ext::timestamp()`
        fn claim(&mut self) -> bool;

        /// Query if `amount_to_borrow` of `borrowed_token` are transfered to borrower
        fn lending_accepted(&mut self) -> bool;
        /// Query if `amount_to_borrow` of `borrowed_token` are transfered to borrower
        fn pledge_accepted(&mut self) -> bool;

        #[event]
        fn LendAccepted(&mut self);
        #[event]
        fn BorrowAccepted(&mut self);

    }

    static BORROWER_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LENDER_KEY: H256 = H256([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static BORROWED_TOKEN_KEY: H256 = H256([4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static SECURITY_TOKEN_KEY: H256 = H256([5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static AMOUNT_TO_BORROW_KEY: H256 = H256([6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static AMOUNT_FOR_SECURITY_KEY: H256 = H256([7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static INTEREST_RATE_KEY: H256 = H256([8,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static ACTIVATION_DEADLINE_KEY: H256 = H256([9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static RETURN_DEADLINE_KEY: H256 = H256([10,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static BORROW_ACCEPTED_KEY: H256 = H256([11,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static LEND_ACCEPTED_KEY: H256 = H256([12,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

    static DIVISOR: U256 = U256([100,0,0,0]);

    pub struct RepoContractInstance {
        storage: Storage
    }

    impl RepoContractInstance {
        pub fn new() -> RepoContractInstance {
            RepoContractInstance {
                storage: Storage::with_capacity(10)
            }
        }
        pub fn read_borrower_address(&mut self) -> Address {
            H256::from(self.storage.read(&BORROWER_KEY)).into()
        }

        pub fn read_lender_address(&mut self) -> Address {
            H256::from(self.storage.read(&LENDER_KEY)).into()
        }

        pub fn read_borrowed_token_address(&mut self) -> Address {
            H256::from(self.storage.read(&BORROWED_TOKEN_KEY)).into()
        }

        pub fn read_security_token_address(&mut self) -> Address {
            H256::from(self.storage.read(&SECURITY_TOKEN_KEY)).into()
        }

        pub fn read_amount_to_borrow(&mut self) -> U256 {
            self.storage.read(&AMOUNT_TO_BORROW_KEY).into()
        }

        pub fn read_security_amount(&mut self) -> U256 {
            self.storage.read(&AMOUNT_FOR_SECURITY_KEY).into()
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
            borrowed_token: Address,
            security_token: Address,
            amount_to_borrow: U256,
            security_amount: U256,
            interest_rate: U256,
            activation_deadline: u64,
            return_deadline: u64) {

            storage::write(&BORROWER_KEY, &H256::from(borrower).into());
            storage::write(&LENDER_KEY, &H256::from(lender).into());
            storage::write(&BORROWED_TOKEN_KEY, &H256::from(borrowed_token).into());
            storage::write(&SECURITY_TOKEN_KEY, &H256::from(security_token).into());
            storage::write(&AMOUNT_TO_BORROW_KEY, &amount_to_borrow.into());
            storage::write(&AMOUNT_FOR_SECURITY_KEY, &security_amount.into());
            storage::write(&INTEREST_RATE_KEY, &interest_rate.into());
            storage::write(&ACTIVATION_DEADLINE_KEY, &U256::from(activation_deadline).into());
            storage::write(&RETURN_DEADLINE_KEY, &U256::from(return_deadline).into());
        }

        fn pledge(&mut self) -> bool {
            // if ext::timestamp() > read_activation_deadline()
            // 1. Do not allow to join if activation deadline has reached
            // 2. Only borrower can pledge
            if ext::timestamp() > self.read_activation_deadline() {
                ext::suicide(&ext::sender());
                return false;
            }
            if ext::sender() != self.read_borrower_address() {
                return false;
            }
            storage::write(&BORROW_ACCEPTED_KEY, &U256::from(1).into());

            return true;
        }

        fn lend(&mut self) -> bool {
            if self.read_lender_acceptance() {
                return true;
            }
            // 1. Do not allow to join if activation deadline has reached
            if ext::timestamp() > self.read_activation_deadline() {
                ext::suicide(&ext::sender());
                return false;
            }
            true
        }
        fn redeem(&mut self) -> bool {
            unimplemented!()
        }

        fn claim(&mut self) -> bool {
            unimplemented!()
        }

        fn pledge_accepted(&mut self) -> bool {
            unimplemented!()
        }

        fn lending_accepted(&mut self) -> bool {
            unimplemented!()
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
            let borrowed_token = Address::from("0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
            let security_token = Address::from("0xcd1722f2947def4cf144679da39c4c32bdc35681");
            let amount_to_borrow: U256 = 10000.into();
            let security_amount: U256 = 50000.into();
            let interest_rate: U256 = 3.into();
            let activation_deadline: u64 = 10;
            let return_deadline: u64 = 20;

            contract.constructor(borrower.clone(), lender.clone(), borrowed_token.clone(), security_token.clone(),
                amount_to_borrow, security_amount, interest_rate, activation_deadline, return_deadline);
            assert_eq!(contract.read_borrower_address(), borrower);
            assert_eq!(contract.read_lender_address(), lender);
            assert_eq!(contract.read_borrowed_token_address(), borrowed_token);
            assert_eq!(contract.read_security_token_address(), security_token);
            assert_eq!(contract.read_amount_to_borrow(), amount_to_borrow);
            assert_eq!(contract.read_security_amount(), security_amount);
            assert_eq!(contract.read_interest_rate(), interest_rate);
            assert_eq!(contract.read_activation_deadline(), activation_deadline);
            assert_eq!(contract.read_return_deadline(), return_deadline);
        }
    );

    test_with_external!(
        ExternalBuilder::new().sender("0xea674fdde714fd979de3edf0f56aa9716b898ec8".into()).timestamp(5).build(),
        should_pledge {
            let mut contract = RepoContractInstance::new();
            let borrower = Address::from("0xea674fdde714fd979de3edf0f56aa9716b898ec8");
            let lender =  Address::from("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1");
            let borrowed_token = Address::from("0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
            let security_token = Address::from("0xcd1722f2947def4cf144679da39c4c32bdc35681");
            let amount_to_borrow: U256 = 10000.into();
            let security_amount: U256 = 50000.into();
            let interest_rate: U256 = 3.into();
            let activation_deadline: u64 = 10;
            let return_deadline: u64 = 20;

            contract.constructor(borrower,
             lender.clone(), borrowed_token.clone(), security_token.clone(),
                amount_to_borrow, security_amount, interest_rate, activation_deadline, return_deadline);

            assert_eq!(contract.pledge(), true);

        }
    );

}
