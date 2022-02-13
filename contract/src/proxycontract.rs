#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

// We need to explicitly import the std alloc crate and `alloc::string::String` as we're in a
// `no_std` environment.
extern crate alloc;

use alloc::string::String;

use casper_contract::contract_api::{runtime, storage};
use casper_types::{runtime_args, ContractPackageHash, RuntimeArgs};

#[no_mangle]
pub extern "C" fn call() {
    // runtime::call_contract(contract_hash, entry_point_name, runtime_args);
    let contract_package_hash: ContractPackageHash = runtime::get_named_arg("packagehash");
    let retvalue: String =
        runtime::call_versioned_contract(contract_package_hash, None, "test", runtime_args! {});
    runtime::put_key("retvalue", storage::new_uref(retvalue).into());
}
