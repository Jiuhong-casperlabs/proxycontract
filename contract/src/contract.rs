#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{CLType, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints};

#[no_mangle]
pub extern "C" fn test() {
    let ret = CLValue::from_t("helloworld").unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entrypoints = EntryPoints::new();

    let entrypoint = EntryPoint::new(
        "test",
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(entrypoint);

    let (contracthash, _contractversion) = storage::new_contract(
        entrypoints,
        None,
        Some("packagehashname".to_string()),
        None,
    );
    runtime::put_key("contract", contracthash.into());
}
