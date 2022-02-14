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

use casper_contract::contract_api::{runtime, storage};
use casper_types::{
    runtime_args, CLType, CLTyped, ContractPackageHash, EntryPoint, EntryPointAccess, EntryPoints,
    Key, Parameter, RuntimeArgs,
};

#[no_mangle]
pub extern "C" fn callme() {
    let packagekey: Key = runtime::get_named_arg("packagekey");
    let hash = packagekey.into_hash().unwrap();
    let contract_package_hash = ContractPackageHash::new(hash);
    let value: String = runtime::get_named_arg("message");
    let retvalue: String = runtime::call_versioned_contract(
        contract_package_hash,
        None,
        "test",
        runtime_args! {
            "message" => value
        },
    );
    runtime::put_key("retvalue", storage::new_uref(retvalue).into());
}

#[no_mangle]
pub extern "C" fn call() {
    // runtime::call_contract(contract_hash, entry_point_name, runtime_args);
    let mut entrypoints = EntryPoints::new();

    let entrypoint1 = EntryPoint::new(
        "callme",
        vec![
            Parameter::new("packagekey", Key::cl_type()),
            Parameter::new("message", String::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        casper_types::EntryPointType::Session,
    );
    entrypoints.add_entry_point(entrypoint1);

    let (contracthash, _contractversion) =
        storage::new_contract(entrypoints, None, Some("proxyhashname".to_string()), None);
	runtime::put_key("proxycontract", contracthash.into());
}
