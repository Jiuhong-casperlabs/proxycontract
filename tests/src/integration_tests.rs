#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_ADDR, DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG,
        DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash, runtime_args, CLValue, ContractPackageHash, Key, Motes, PublicKey,
        RuntimeArgs, SecretKey, StoredValue, U512,
    };

    const MY_ACCOUNT: [u8; 32] = [7u8; 32];
    const CONTRACT_WASM: &str = "contract.wasm";
    const PROXYCONTRACT_WASM: &str = "proxycontract.wasm";

    #[test]
    fn should_store_hello_world() {
        // Create keypair.
        let secret_key = SecretKey::ed25519_from_bytes(MY_ACCOUNT).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);
        // Create a GenesisAccount.
        let account = GenesisAccount::account(
            public_key,
            Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
            None,
        );

        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        genesis_config.ee_config_mut().push_account(account);

        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );
        // The test framework checks for compiled Wasm files in '<current working dir>/wasm'.  Paths
        // relative to the current working dir (e.g. 'wasm/contract.wasm') can also be used, as can
        // absolute paths.

        // install contract.wasm
        let session_code = PathBuf::from(CONTRACT_WASM);
        let session_args = runtime_args! {};

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {
                ARG_AMOUNT => *DEFAULT_PAYMENT
            })
            .with_session_code(session_code, session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item).build();

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();

        // deploy the contract.
        builder.exec(execute_request).commit().expect_success();


        // install PROXYCONTRACT
        let proxy_installer_session_code = PROXYCONTRACT_WASM;
        let proxy_installer_session_args = runtime_args! {};
        let installer_payment_args = runtime_args! {
            ARG_AMOUNT => *DEFAULT_PAYMENT
        };
        let deploy_item2 = DeployItemBuilder::new()
            .with_empty_payment_bytes(installer_payment_args)
            .with_session_code(proxy_installer_session_code, proxy_installer_session_args)
            .with_authorization_keys(&[account_addr])
            .with_address(account_addr)
            .build();

        let execute_request2 = ExecuteRequestBuilder::from_deploy_item(deploy_item2).build();

        builder.exec(execute_request2).commit().expect_success();
        // ~~~~~~~~~~
        //get account
        let account = builder
            .query(None, Key::Account(account_addr), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        // get named_keys
        let named_keys = account.named_keys();
        println!("named_keys are {:?}", named_keys);

		//get stored package key
		let stored_package_key = account
		.named_keys()
		.get("packagehashname")
		.expect("should have stored uref").clone();

        // call entrypoint 'callme' of proxycontract
        let deploy = DeployItemBuilder::new()
            .with_address(account_addr)
            .with_stored_versioned_contract_by_name(
                "proxyhashname",
                None,
                "callme",
                runtime_args! {
                    "packagekey".to_string() => stored_package_key,
                    "message".to_string() => "helloworld",
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[account_addr])
            .with_deploy_hash([42; 32])
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        builder.exec(execute_request).commit().expect_success();
        // `````````````
        // query stored value under account
        let account = builder
            .get_account(*DEFAULT_ACCOUNT_ADDR)
            .expect("should have account");

        let retvaluekey = *account
            .named_keys()
            .get("retvalue")
            .expect("version key should exist");

        let retvalue = builder
            .query(None, retvaluekey, &[])
            .expect("helloworld should exist");

        // make assertions
        assert_eq!(
            retvalue,
            StoredValue::CLValue(CLValue::from_t("helloworld".to_string()).unwrap()),
            "should be helloworld"
        );
    }
}

fn main() {
    panic!("Execute \"cargo test\" to test the contract, not \"cargo run\".");
}
