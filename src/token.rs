// Contract doesn't use standard library
#![no_std]
#![feature(wasm_import_memory)]
#![wasm_import_memory]

extern crate pwasm_std;
extern crate pwasm_abi;
extern crate pwasm_token_contract;

use pwasm_abi::eth::EndpointInterface;

/// The main function receives a pointer for the call descriptor.
#[no_mangle]
pub fn call(desc: *mut u8) {
    // pwasm_std::parse_args parses the call descriptor into arguments and result pointers
    let (args, result) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = pwasm_token_contract::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
    // Args is an Solidity-compatible abi call: first 4 bytes are the Method ID of keccak hash of function signature
    // followed by sequence of arguments packed into chunks of 32 bytes.
    // Read http://solidity.readthedocs.io/en/develop/abi-spec.html#formal-specification-of-the-encoding for details
    result.done(endpoint.dispatch(&args));
}

#[no_mangle]
pub fn deploy(desc: *mut u8) {
    let (args, _) = unsafe { pwasm_std::parse_args(desc) };
    let mut endpoint = pwasm_token_contract::Endpoint::new(pwasm_token_contract::TokenContractInstance{});
    endpoint.dispatch_ctor(&args);
}
