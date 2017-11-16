extern crate pwasm_std;


use pwasm_std::{storage, ext, logger};

pub trait External {

	/// Invoked when contract is requesting storage_read extern
	fn storage_read(&mut self, key: &H256) -> Result<[u8; 32], Error>;

	/// Invoked when contract is requesting storage_write extern
	fn storage_write(&mut self, key: &H256, value: &[u8; 32]) -> Result<(), Error>;

	/// Invoked when contract is requesting balance extern
	fn balance(&mut self, address: &Address) -> U256;

	/// Invoked when contract is requesting suicide extern
	fn suicide(&mut self, refund: &Address);

	/// Invoked when contract is requesting create extern
	fn create(&mut self, endowment: U256, code: &[u8]) -> Result<Address, Error>;

	/// Invoked when contract is requesting regular call (ccall) extern
	fn call(&mut self, address: &Address, val: U256, input: &[u8], _result: &mut [u8]) -> Result<(), Error>;

	/// Invoked when contract is requesting delegate call (dcall) extern
	fn call_code(&mut self, address: &Address, input: &[u8], result: &mut [u8]) -> Result<(), Error>;

	/// Invoked when contract is requesting static call (ccall) extern
	fn static_call(&mut self, address: &Address, input: &[u8], result: &mut [u8]) -> Result<(), Error>;

	/// Invoked when contract is requesting debug message extern
	fn debug_log(&mut self, msg: String);

	/// Invoked when contract is requesting blockhash extern
	fn blockhash(&mut self, number: u64) -> Result<H256, Error>;

	/// Invoked when contract is requesting coinbase extern
	fn coinbase(&mut self) -> Address;

	/// Invoked when contract is requesting timestamp extern
	fn timestamp(&mut self) -> u64;

	/// Invoked when contract is requesting blocknumber extern
	fn blocknumber(&mut self) -> u64;

	/// Invoked when contract is requesting difficulty extern
	fn difficulty(&mut self) -> U256;

	/// Invoked when contract is requesting gas_limit extern
	fn gas_limit(&mut self) -> U256;

	/// Invoked when contract is requesting sender data
	fn sender(&mut self) -> Address;

	/// Invoked when contract is requesting origin data
	fn origin(&mut self) -> Address;

	/// Invoked when contract is requesting value data
	fn value(&mut self) -> U256;

	/// Invoked when contract is requesting contract address
	fn address(&mut self) -> Address;
}
struct NativeExternal;

pub impl External for NativeExternal {

	fn storage_read(&mut self, key: &H256) -> Result<[u8; 32], Error>  {
		storage::read(key)
	}

	fn storage_write(&mut self, key: &H256, value: &[u8; 32]) -> Result<(), Error> {
		storage::write(key, value)
	}

	fn balance(&mut self, address: &Address) -> U256 {
		ext::balance()
	}

	fn suicide(&mut self, refund: &Address) {
		ext::suicide(refund);
	}

	fn create(&mut self, endowment: U256, code: &[u8]) -> Result<Address, Error> {
		ext::create(endowment, code)
	}

	fn call(&mut self, address: &Address, val: U256, input: &[u8], _result: &mut [u8]) -> Result<(), Error> {
		ext::call(address, val, input, result)
	}

	fn call_code(&mut self, address: &Address, input: &[u8], result: &mut [u8]) -> Result<(), Error> {
		ext::call_code(address, input, result)
	}

	fn static_call(&mut self, address: &Address, input: &[u8], result: &mut [u8]) -> Result<(), Error> {
		ext::static_call(address, input, result)
	}

	fn debug_log(&mut self, msg: String) {
		ext::logger(msg);
	}

	fn blockhash(&mut self, number: u64) -> Result<H256, Error> {
		ext::blockhash(number)
	}

	fn coinbase(&mut self) -> Address {
		ext::coinbase()
	}

	fn timestamp(&mut self) -> u64 {
		ext::timestamp()
	}

	fn blocknumber(&mut self) -> u64 {
		ext::blocknumber()
	}

	fn difficulty(&mut self) -> U256 {
		ext::difficulty()
	}

	fn gas_limit(&mut self) -> U256 {
		ext::gas_limit()
	}

	fn sender(&mut self) -> Address {
		ext::sender()
	}

	fn origin(&mut self) -> Address {
		ext::origin()
	}

	fn value(&mut self) -> U256 {
		ext::value()
	}

	fn address(&mut self) -> Address {
		ext::address()
	}
}
