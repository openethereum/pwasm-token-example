#![feature(proc_macro)]
#![feature(alloc)]

// Conract doesn't need standard library and the `main` function.
// `cargo test` requires `std` and it provided the `main` which is why the "std" feature should be turned on for `cargo test`
#![cfg_attr(not(feature="std"), no_main)]
#![cfg_attr(not(feature="std"), no_std)]

extern crate tiny_keccak;
extern crate alloc;
extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;

use pwasm_abi::eth::EndpointInterface;

pub mod contract {
    #![allow(non_snake_case)]
    use alloc::vec::Vec;

    use tiny_keccak::Keccak;
    use pwasm_std::{storage, ext};
    use pwasm_std::hash::{Address, H256};
    use pwasm_std::bigint::U256;

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
        fn constructor(&mut self, _total_supply: U256);
        fn balanceOf(&mut self, _owner: Address) -> U256;
        fn transfer(&mut self, _to: Address, _amount: U256) -> bool;
        fn totalSupply(&mut self) -> U256;
        fn transferFrom(&mut self, _from: Address, _to: Address, _amount: U256) -> bool;
        //fn approve(&mut self, _spender: Address, _value: U256) -> bool;
        fn allowance(&mut self, _owner: Address, _spender: Address) -> U256;

        #[event]
        fn Transfer(&mut self, indexed_from: Address, indexed_to: Address, value: U256);
    }

    static TOTAL_SUPPLY_KEY: H256 = H256([2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
    static OWNER_KEY: H256 = H256([3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);


    fn read_balance_of(_owner: &Address) -> U256 {
        storage::read(&balance_key(_owner)).into()
    }

    fn read_allowance(key: &H256) -> U256 {
        storage::read(&key).into()
    }

    fn allowance_key(owner: &Address, spender: &Address) -> H256 {
        let mut keccak = Keccak::new_keccak256();
        let mut res = H256::new();
        keccak.update("allowance_key".as_ref());
        keccak.update(owner.as_ref());
        keccak.update(spender.as_ref());
        keccak.finalize(&mut res);
        res
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
            storage::write(&TOTAL_SUPPLY_KEY, &total_supply.into());
            // Give all tokens to the contract owner
            storage::write(&balance_key(&sender), &total_supply.into());
            // Set the contract owner
            storage::write(&OWNER_KEY, &H256::from(sender).into());
        }

        /// Returns the current balance for some address.
        fn balanceOf(&mut self, owner: Address) -> U256 {
            read_balance_of(&owner)
        }

        /// Transfer funds
        fn transfer(&mut self, to: Address, amount: U256) -> bool {
            let sender = ext::sender();
            let senderBalance = read_balance_of(&sender);
            let recipientBalance = read_balance_of(&to);
            if amount == 0.into() || senderBalance < amount {
                false
            } else {
                storage::write(&balance_key(&sender), &(senderBalance - amount).into());
                storage::write(&balance_key(&to), &(recipientBalance + amount).into());
                self.Transfer(sender, to, amount);
                true
            }
        }

        fn allowance(&mut self, owner: Address, spender: Address) -> U256 {
            storage::read(&allowance_key(&owner, &spender)).into()
        }

        fn transferFrom(&mut self, from: Address, to: Address, amount: U256) -> bool {
            let fromBalance = read_balance_of(&from);
            let recipientBalance = read_balance_of(&to);
            let a_key = allowance_key(&from, &to);
            let allowed = read_allowance(&a_key);
            if  allowed < amount || amount == 0.into() || fromBalance < amount {
                false
            } else {
                storage::write(&a_key, &(allowed - amount).into());
                storage::write(&balance_key(&from), &(fromBalance - amount).into());
                storage::write(&balance_key(&to), &(recipientBalance + amount).into());
                self.Transfer(from, to, amount);
                true
            }
        }

        /// Returns total amount of tokens
        fn totalSupply(&mut self) -> U256 {
            storage::read(&TOTAL_SUPPLY_KEY).into()
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
    use self::pwasm_test::{External, ExternalBuilder, ExternalInstance, get_external, set_external};
    use pwasm_std::bigint::U256;
    use pwasm_std::hash::{Address};

    test_with_external!(
        ExternalBuilder::new()
            .storage([1,0,0,0,0,0,0,0,0,0,0,0,
                            31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31].into(), U256::from(100000).into())
            .build(),
        balanceOf_should_return_balance {
            let address = Address::from([31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31]);
            let mut contract = TokenContractInstance{};
            assert_eq!(contract.balanceOf(address), 100000.into())
        }
    );

    test_with_external!(
        ExternalBuilder::new().build(),
        totalSupply_should_return_total_supply_contract_was_initialized_with {
            let mut contract = TokenContractInstance{};
            let total_supply = 42.into();
            contract.constructor(total_supply);
            assert_eq!(contract.totalSupply(), total_supply);
        }
    );

    test_with_external!(
        ExternalBuilder::new().build(),
        should_succeed_in_creating_max_possible_amount_of_tokens {
            let mut contract = TokenContractInstance{};
            // set total supply to maximum value of an unsigned 256 bit integer
            let total_supply = U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap();
            assert_eq!(total_supply, U256::max_value());
            contract.constructor(total_supply);
            assert_eq!(contract.totalSupply(), total_supply);
        }
    );

    test_with_external!(
        ExternalBuilder::new().build(),
        should_initially_give_the_total_supply_to_the_creator {
            let mut contract = TokenContractInstance{};
            let total_supply = 10000.into();
            contract.constructor(total_supply);
            assert_eq!(
                contract.balanceOf(get_external::<ExternalInstance>().sender()),
                total_supply);
        }
    );

    #[test]
    fn should_succeed_transfering_1000_from_owner_to_another_address() {
        let mut contract = TokenContractInstance{};

        let owner_address = Address::from("0xea674fdde714fd979de3edf0f56aa9716b898ec8");
        let sam_address = Address::from("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1");

        set_external(Box::new(ExternalBuilder::new()
            .sender(owner_address.clone())
            .build()));

        let total_supply = 10000.into();
        contract.constructor(total_supply);

        assert_eq!(contract.balanceOf(owner_address), total_supply);

        assert_eq!(contract.transfer(sam_address, 1000.into()), true);
        assert_eq!(get_external::<ExternalInstance>().logs().len(), 1);
        assert_eq!(get_external::<ExternalInstance>().logs()[0].topics.as_ref(), &[
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".into(),
            "0x000000000000000000000000ea674fdde714fd979de3edf0f56aa9716b898ec8".into(),
            "0x000000000000000000000000db6fd484cfa46eeeb73c71edee823e4812f9e2e1".into()]);
        assert_eq!(get_external::<ExternalInstance>().logs()[0].data.as_ref(), &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 232]);
        assert_eq!(contract.balanceOf(owner_address), 9000.into());
        assert_eq!(contract.balanceOf(sam_address), 1000.into());
    }

    #[test]
    fn should_return_false_transfer_not_sufficient_funds() {
        set_external(Box::new(ExternalBuilder::new()
            .build()));
        let mut contract = TokenContractInstance{};
        contract.constructor(10000.into());
        assert_eq!(contract.transfer("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1".into(), 50000.into()), false);
        assert_eq!(contract.balanceOf(::pwasm_std::ext::sender()), 10000.into());
        assert_eq!(contract.balanceOf("0xdb6fd484cfa46eeeb73c71edee823e4812f9e2e1".into()), 0.into());
    }
}
