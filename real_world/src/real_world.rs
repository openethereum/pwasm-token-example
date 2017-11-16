extern crate wasm;
extern crate vm;
extern crate ethcore_bigint;
extern crate ethcore_logger;
extern crate byteorder;

use std::sync::Arc;
use vm::{Vm, GasLeft, ActionParams};
use vm::tests::FakeExt;
use wasm::WasmInterpreter;
use ethcore_bigint::prelude::{U256, H160};
use byteorder::{BigEndian, ByteOrder};

static CONTRACT: &'static [u8] = include_bytes!("../../compiled/token.wasm");

fn wasm_interpreter() -> WasmInterpreter {
    WasmInterpreter::new().expect("wasm interpreter to create without errors")
}

fn address1() -> H160 {
    "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap()
}

fn address2() -> H160 {
    "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f".parse().unwrap()
}

fn address3() -> H160 {
    "0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d".parse().unwrap()
}

fn construct_contract(ext: &mut vm::Ext, total_supply: U256) -> Vec<u8> {
    let mut data = vec![0; 32];
    total_supply.to_big_endian(&mut data);

    let mut params = ActionParams::default();
    params.sender = address2();
    params.address = address1();
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

    let wasm_binary = construct_contract(&mut ext, U256::from(100000000));

    let mut data = vec![0u8; 4];
    BigEndian::write_u32(&mut data, 404098525u32);

    let mut params = ActionParams::default();
    params.sender = address2();
    params.address = address1();
    params.gas = U256::from(100_000);
    params.data = Some(data);

    params.code = Some(Arc::new(wasm_binary));

    let mut interpreter = wasm_interpreter();
    let result_data = match interpreter.exec(params, &mut ext) {
        Ok(GasLeft::NeedsReturn { data: result, .. } ) => result.to_vec(),
        _ => panic!(),
    };

    assert_eq!(U256::from_big_endian(&result_data), U256::from(100000000));
}

#[test]
fn transfer() {
    ::ethcore_logger::init_log();

    let mut ext = FakeExt::new();

    let wasm_binary = construct_contract(&mut ext, U256::from(100000000));

    let mut data = vec![0u8; 68];
    BigEndian::write_u32(&mut data[0..4], 2835717307);
    data[16..36].copy_from_slice(&*address3());
    let val1 = U256::from(1000000);
    val1.to_big_endian(&mut data[36..68]);

    let mut params = ActionParams::default();
    params.sender = address2();
    params.gas = U256::from(100_000);
    params.data = Some(data);
    params.code = Some(Arc::new(wasm_binary));

    let mut interpreter = wasm_interpreter();
    let _result = match interpreter.exec(params, &mut ext) {
        Ok(GasLeft::NeedsReturn { data: result, .. } ) => result.to_vec(),
        _ => panic!(),
    };

    assert_eq!(
        ext.store.remove(&"0100000000000000000000000d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d".parse().unwrap()).unwrap(),
        U256::from(1000000).into()
    );
}
