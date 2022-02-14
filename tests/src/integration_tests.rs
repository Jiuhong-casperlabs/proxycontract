#[cfg(test)]
mod tests {
    use casper_engine_test_support::{
        DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
        DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
        DEFAULT_PAYMENT,
    };
    use casper_execution_engine::core::engine_state::{
        run_genesis_request::RunGenesisRequest, GenesisAccount,
    };
    use casper_types::{
        account::AccountHash, runtime_args, CLValue, Key, Motes, PublicKey, RuntimeArgs, SecretKey,
        StoredValue, U512,
    };

    const CONTRACT_WASM: &str = "contract.wasm";
    const PROXYCONTRACT_WASM: &str = "proxycontract.wasm";

    #[test]
    fn test1() {
        let secret_key = SecretKey::ed25519_from_bytes([1u8; 32]).unwrap();
        let public_key = PublicKey::from(&secret_key);
        let account_address = AccountHash::from(&public_key);

        // Make this account a genesis account (one which exists at network startup) and a
        // genesis request for the execution engine.
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

        let mut test_builder = InMemoryWasmTestBuilder::default();
        test_builder.run_genesis(&run_genesis_request).commit();

        // ========= install contract start========= //
        let exec_request_1 =
            ExecuteRequestBuilder::standard(account_address, CONTRACT_WASM, runtime_args! {})
                .build();

        // ========= install contract end========= //

        // ========= install proxy contract start========= //

        let exec_request_2 =
            ExecuteRequestBuilder::standard(account_address, PROXYCONTRACT_WASM, runtime_args! {})
                .build();

        test_builder.exec(exec_request_1).expect_success().commit();

        test_builder.exec(exec_request_2).expect_success().commit();
        // ========= install proxy contract end========= //

        //get account
        let account = test_builder
            .query(None, Key::Account(account_address), &[])
            .expect("should query account")
            .as_account()
            .cloned()
            .expect("should be account");

        //get contract package key
        let contract_package_key = *account
            .named_keys()
            .get("packagehashname")
            .expect("should have packagehashname");

        // get named_keys
        // let named_keys = account
        //     .named_keys().clone();
        // println!("named keys are {:?}",named_keys);

        //call entrypoint callme of proxy contract

        let deploy = DeployItemBuilder::new()
            .with_address(account_address)
            .with_stored_versioned_contract_by_name(
                "proxyhashname",
                None,
                "callme",
                runtime_args! {
                    "packagekey" => contract_package_key,
                    "message" => String::from("helloworld"),
                },
            )
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT, })
            .with_authorization_keys(&[account_address])
            .with_deploy_hash([42; 32])
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy).build();
        test_builder.exec(execute_request).commit().expect_success();

        // query stored value under account
        let account = test_builder
            .get_account(account_address)
            .expect("should have account");

        let retvaluekey = *account
            .named_keys()
            .get("retvalue")
            .expect("version key should exist");

        let retvalue = test_builder
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
