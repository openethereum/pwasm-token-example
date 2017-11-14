extern crate wasm;
extern crate vm;
extern crate ethcore_bigint;
extern crate ethcore_logger;
extern crate byteorder;

use std::sync::Arc;
use vm::{Vm, GasLeft, ActionParams};
use vm::tests::FakeExt;
use wasm::WasmInterpreter;
use ethcore_bigint::prelude::U256;
use byteorder::{BigEndian, ByteOrder};

static CONTRACT: &'static [u8] = include_bytes!("../../compiled/token.wasm");

fn wasm_interpreter() -> WasmInterpreter {
    WasmInterpreter::new().expect("wasm interpreter to create without errors")
}

fn construct_contract(ext: &mut vm::Ext, total_supply: U256) -> Vec<u8> {
    let mut data = vec![0; 32];
    total_supply.to_big_endian(&mut data);

    let mut params = ActionParams::default();
    params.address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap(); // totally random address
    params.gas = U256::from(100_000_000);
    params.data = Some(data);
    params.code = Some(Arc::new(CONTRACT.to_vec()));

    let mut interpreter = wasm_interpreter();
    let wasm_binary = match interpreter.exec(params, ext) {
        Ok(GasLeft::NeedsReturn { data: result, .. } ) => result.to_vec(),
        _ => panic!(),
    };

    assert_eq!(&wasm_binary[0..4], b"\0asm");

    wasm_binary
}

#[test]
fn simple_test() {
    ::ethcore_logger::init_log();

    let mut ext = FakeExt::new();

    let total_supply = U256::from(1);
    let wasm_binary = construct_contract(&mut ext, total_supply);

    let mut data = vec![0u8; 4];
    BigEndian::write_u32(&mut data, 404098525u32);

    let mut params = ActionParams::default();
    params.address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap(); // totally random address
    params.gas = U256::from(100_000);
    params.data = Some(data);
    params.code = Some(Arc::new(wasm_binary));

    let mut interpreter = wasm_interpreter();
    let result_data = match interpreter.exec(params, &mut ext) {
        Ok(GasLeft::NeedsReturn { data: result, .. } ) => result.to_vec(),
        _ => panic!(),
    };

    assert_eq!(U256::from_big_endian(&result_data), total_supply);
}
