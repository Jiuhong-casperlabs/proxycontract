#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec,
};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

#[no_mangle]
pub extern "C" fn test() {
    let message: String = runtime::get_named_arg("message");
    let ret = CLValue::from_t(message).unwrap_or_revert();
    runtime::ret(ret);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entrypoints = EntryPoints::new();

    let entrypoint = EntryPoint::new(
        "test",
        vec![Parameter::new("message", String::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    entrypoints.add_entry_point(entrypoint);

    let (contracthash, _contractversion) =
        storage::new_contract(entrypoints, None, Some("packagehashname".to_string()), None);
    runtime::put_key("contract", contracthash.into());

}
