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

    // Following imports are used by generated eth_abi code
    use pwasm_std::ext::{call, log};

    use pwasm_abi_derive::eth_abi;

    // TokenContract is an interface definition of a contract.
    // The current example covers the minimal subset of ERC20 token standard.
    // eth_abi macro parses an interface (trait) definition of a contact and generates
    // two structs: Endpoint and Client.
    //
    // Endpoint is an entry point for contract calls.
    // eth_abi macro generates a table of Method IDs corresponding with every method signature defined in the trait
    // and defines it statically in the generated code.
    // Scroll down at "pub fn call(desc: *mut u8)" to see how
    // Endpoint instantiates with a struct TokenContractInstance which implements the trait definition.
    //
    // Client is a struct which is useful for call generation to a deployed contract. For example:
    // ```
    //     let mut client = Client::new(contactAddress);
    //     let balance = client
    //        .value(someValue) // you can attach some value for a call optionally
    //        .balanceOf(someAddress);
    // ```
    // Will generate a Solidity-compatible call for the contract, deployed on `contactAddress`.
    // Then it invokes pwasm_std::ext::call on `contactAddress` and returns the result.
    #[eth_abi(Endpoint, Client)]
    pub trait TokenContract {
        fn constructor(&mut self, total_supply: U256);
        fn balanceOf(&mut self, _owner: Address) -> U256;
        fn transfer(&mut self, _to: Address, _amount: U256) -> bool;
        fn totalSupply(&mut self) -> U256;

        #[event]
        fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, value: U256);
    }

    static TOTAL_SUPPLY_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static OWNER_KEY: H256 = H256([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

    fn balance_of(owner: &Address) -> U256 {
        storage::read(&balance_key(owner)).unwrap_or([0u8;32]).into()
    }

    // Generates a balance key for some address.
    // Used to map balances with their owners.
    fn balance_key(address: &Address) -> H256 {
        let mut key = H256::from(address);
        key[0] = 1; // just a naiive "namespace";
        key
    }

    pub struct TokenContractInstance;

    impl TokenContract for TokenContractInstance {

        /// A contract constructor implementation.
        fn constructor(&mut self, total_supply: U256) {
            let sender = ext::sender();
            // Set up the total supply for the token
            storage::write(&TOTAL_SUPPLY_KEY, &total_supply.into()).unwrap();
            // Give all tokens to the contract owner
            storage::write(&balance_key(&sender), &total_supply.into()).unwrap();
            // Set the contract owner
            storage::write(&OWNER_KEY, &H256::from(sender).into()).unwrap();
        }

        /// Returns the current balance for some address.
        fn balanceOf(&mut self, owner: Address) -> U256 {
            balance_of(&owner)
        }

        /// Transfer funds
        fn transfer(&mut self, to: Address, amount: U256) -> bool {
            let sender = ext::sender();
            let mut senderBalance = balance_of(&sender);
            let mut recipientBalance = balance_of(&to);
            if amount == 0.into() || senderBalance < amount {
                false
            } else {
                senderBalance = senderBalance - amount;
                recipientBalance = recipientBalance + amount;
                storage::write(&balance_key(&sender), &senderBalance.into()).unwrap();
                storage::write(&balance_key(&to), &recipientBalance.into()).unwrap();
                self.Transfer(sender, to, amount);
                true
            }
        }

        /// Returns total amount of tokens
        fn totalSupply(&mut self) -> U256 {
            storage::read(&TOTAL_SUPPLY_KEY).unwrap_or([0u8; 32]).into()
        }
    }
}

/// The main function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    // pwasm_std::parse_args parses the call descriptor into arguments and result pointers
    // Args is an Solidity-compatible abi call: first 4 bytes are the Method ID of keccak hash of function signature
    // followed by sequence of arguments packed into chunks of 32 bytes.
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = contract::Endpoint::new(contract::TokenContractInstance{});
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = contract::Endpoint::new(contract::TokenContractInstance{});
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
    use self::pwasm_test::{External, Error};
    use self::std::collections::HashMap;
    use pwasm_std::bigint::U256;
    use pwasm_std::hash::{Address, H256};

    struct DummyExternal {
        storage: HashMap<H256, [u8; 32]>
    }

    impl DummyExternal {
        fn new() -> Self {
            let mut storage = HashMap::new();
            storage.insert([1,0,0,0,0,0,0,0,0,0,0,0,
                            31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31].into(), U256::from(100000).into());
            DummyExternal {
                storage: storage
            }
        }
    }

    const SENDER_ADDRESS: &str = "0x16a0772b17ae004e6645e0e95bf50ad69498a34e";

    impl External for DummyExternal {
        fn storage_read(&mut self, key: &H256) -> Result<[u8; 32], Error> {
            if let Some(value) = self.storage.get(key) {
                Ok(value.clone())
            } else {
                Err(Error)
            }
        }
        fn storage_write(&mut self, key: &H256, value: &[u8; 32]) -> Result<(), Error> {
            self.storage.insert(*key, value.clone());
            Ok(())
        }
        fn sender(&mut self) -> Address {
            SENDER_ADDRESS.into()
        }
    }

    test_with_external!(
        DummyExternal::new(),
        balanceOf_should_return_balance {
            let address = Address::from([31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31]);
            let mut contract = TokenContractInstance{};
            assert_eq!(contract.balanceOf(address), 100000.into())
        }
    );
}
