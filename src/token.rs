#![feature(proc_macro)]
#![feature(alloc)]

#![cfg_attr(not(feature="std"), no_main)]
#![cfg_attr(not(feature="std"), no_std)]

#[cfg(feature="std")]
extern crate core;

#[cfg(feature="std")]
extern crate alloc;

extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_abi_derive;

#[macro_use]
extern crate pwasm_test;

use alloc::borrow::Cow;

use pwasm_std::{storage, ext};
use pwasm_std::hash::{Address, H256};
use pwasm_std::bigint::U256;

use pwasm_abi_derive::eth_dispatch;

#[allow(non_snake_case)]
// #[eth_dispatch(Endpoint)]
pub trait TokenContract {
    // fn ctor(&self);
	fn balanceOf(&self, _owner: Address) -> U256;
	fn transfer(&self, _to: Address, _amount: U256) -> bool;
    // fn totalSupply(&self) -> U256;
}

struct TokenContractInstance;

#[allow(non_snake_case)]
impl TokenContract for TokenContractInstance {
    // fn ctor(&mut self, total_supply U256) {

    // }
    fn balanceOf(&self, _owner: Address) -> U256 {
        balanceOf(&_owner)
    }
    fn transfer(&self, _to: Address, _amount: U256) -> bool {
        let sender = ext::sender();
        let mut senderBalance = balanceOf(&sender);
        let mut recipientBalance = balanceOf(&_to);
        if _amount == 0.into() || senderBalance < _amount {
            false
        } else {
            senderBalance = senderBalance - _amount;
            recipientBalance = recipientBalance + _amount;
            storage::write(&balance_key(&sender), &senderBalance.into()).unwrap();
            storage::write(&balance_key(&_to), &recipientBalance.into()).unwrap();
            true
        }
    }
}

fn balanceOf(_owner: &Address) -> U256 {
    storage::read(&balance_key(_owner)).unwrap_or([0u8;32]).into()
}

fn balance_key(address: &Address) -> H256 {
    let mut key = H256::from(address);
    key[0] = 1; // just a naiive "namespace";
    key
}

// myContract.methods.myMethod([param1[, param2[, ...]]]).encodeABI()
// new web3.eth.Contract(jsonInterface[, address][, options])

#[no_mangle]
pub fn call(desc: *mut u8) {
    // let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    // let mut endpoint = Endpoint::new(TokenContractInstance{});
    // result.done(endpoint.dispatch(args));
}

#[no_mangle]
pub fn create(desc: *mut u8) {
    // let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    // let mut endpoint = Endpoint::new(TokenContractInstance{});
    // result.done(endpoint.dispatch(args));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pwasm_test::{External, Error};
    use std::collections::HashMap;
    test_with_external!(
        DummyExternal: impl External for DummyExternal {
            fn storage(&mut self) -> HashMap<H256, [u8; 32]> {
                let mut storage = HashMap::new();
                storage.insert([1,0,0,0,0,0,0,0,0,0,0,0,
                                31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31].into(), U256::from(100000).into());
                storage
            }
            fn storage_write(&mut self, _key: &H256, _value: &[u8; 32]) -> Result<(), Error> {
                Ok(())
            }
        }
        check_balance {
            let address = Address::from([31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31]);
            let contract = TokenContractInstance{};
            assert_eq!(contract.balanceOf(address), 100000.into())
        }
    );
}
