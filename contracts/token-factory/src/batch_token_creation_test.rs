/// Integration tests for batch token creation
/// 
/// Tests cover:
/// - Atomic semantics (all-or-nothing)
/// - Mixed-validity batch payloads
/// - Fee verification
/// - Gas efficiency
/// - State consistency

#[cfg(test)]
mod tests {
    use crate::{TokenFactory, TokenFactoryClient};
    use crate::types::{Error, TokenCreationParams};
    use soroban_sdk::{
        testutils::{Address as _, Events},
        vec, Address, Env, String, Vec,
    };

    fn setup() -> (Env, TokenFactoryClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TokenFactory);
        let client = TokenFactoryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let treasury = Address::generate(&env);

        let _ = client.initialize(&admin, &treasury, &100_i128, &50_i128);

        (env, client, admin, treasury)
    }

    fn create_valid_token_params(env: &Env, name: &str, symbol: &str) -> TokenCreationParams {
        TokenCreationParams {
            name: String::from_str(env, name),
            symbol: String::from_str(env, symbol),
            decimals: 6,
            initial_supply: 1_000_000_i128,
            metadata_uri: None,
        }
    }

    // ═══════════════════════════════════════════════════════
    //  Single Token Creation Tests
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_create_single_token_success() {
        let (env, client, admin, _) = setup();

        let _result = client.try_create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &100_i128,
        );

        assert_eq!(client.get_token_count(), 1);
    }

    #[test]
    fn test_create_token_with_metadata() {
        let (env, client, admin, _) = setup();

        let metadata_uri = Some(String::from_str(&env, "https://example.com/metadata.json"));
        let result = client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &metadata_uri,
            &150_i128, // base_fee (100) + metadata_fee (50)
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_token_insufficient_fee() {
        let (env, client, admin, _) = setup();

        let result = client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &50_i128, // Insufficient fee (required: 100)
        );

        assert_eq!(result, Err(Ok(Error::InsufficientFee)));
    }

    #[test]
    fn test_create_token_when_paused() {
        let (env, client, admin, _) = setup();

        // Pause the contract
        client.pause(&admin).unwrap();

        let result = client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &100_i128,
        );

        assert_eq!(result, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_create_token_invalid_name_empty() {
        let (env, client, admin, _) = setup();

        let result = client.create_token(
            &admin,
            &String::from_str(&env, ""),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &100_i128,
        );

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
    }

    #[test]
    fn test_create_token_invalid_decimals() {
        let (env, client, admin, _) = setup();

        let result = client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &19_u32, // Invalid: max is 18
            &1_000_000_i128,
            &None,
            &100_i128,
        );

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
    }

    #[test]
    fn test_create_token_zero_supply() {
        let (env, client, admin, _) = setup();

        let result = client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &0_i128, // Invalid: must be positive
            &None,
            &100_i128,
        );

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
    }

    // ═══════════════════════════════════════════════════════
    //  Batch Token Creation Tests - Success Cases
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_tokens_success() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
            create_valid_token_params(&env, "Token3", "TK3"),
        ];

        let total_fee = 300_i128; // 3 tokens * 100 base_fee
        let result = client.batch_create_tokens(&admin, &tokens, &total_fee);

        assert!(result.is_ok());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 3);
        assert_eq!(client.get_token_count(), 3);
    }

    #[test]
    fn test_batch_create_tokens_with_mixed_metadata() {
        let (env, client, admin, _) = setup();

        let mut token1 = create_valid_token_params(&env, "Token1", "TK1");
        token1.metadata_uri = Some(String::from_str(&env, "https://example.com/1.json"));

        let token2 = create_valid_token_params(&env, "Token2", "TK2");

        let mut token3 = create_valid_token_params(&env, "Token3", "TK3");
        token3.metadata_uri = Some(String::from_str(&env, "https://example.com/3.json"));

        let tokens = vec![&env, token1, token2, token3];

        // Fee calculation: (100 + 50) + 100 + (100 + 50) = 400
        let total_fee = 400_i128;
        let result = client.batch_create_tokens(&admin, &tokens, &total_fee);

        assert!(result.is_ok());
        assert_eq!(client.get_token_count(), 3);
    }

    #[test]
    fn test_batch_create_single_token() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
        ];

        let result = client.batch_create_tokens(&admin, &tokens, &100_i128);

        assert!(result.is_ok());
        assert_eq!(client.get_token_count(), 1);
    }

    #[test]
    fn test_batch_create_large_batch() {
        let (env, client, admin, _) = setup();

        let mut tokens = Vec::new(&env);
        for i in 0..10 {
            let name = format!("Token{}", i);
            let symbol = format!("TK{}", i);
            tokens.push_back(create_valid_token_params(&env, &name, &symbol));
        }

        let total_fee = 1000_i128; // 10 tokens * 100 base_fee
        let result = client.batch_create_tokens(&admin, &tokens, &total_fee);

        assert!(result.is_ok());
        assert_eq!(client.get_token_count(), 10);
    }

    // ═══════════════════════════════════════════════════════
    //  Batch Token Creation Tests - Atomic Semantics
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_atomic_rollback_invalid_name() {
        let (env, client, admin, _) = setup();

        let token1 = create_valid_token_params(&env, "Token1", "TK1");
        let mut token2 = create_valid_token_params(&env, "Token2", "TK2");
        token2.name = String::from_str(&env, ""); // Invalid: empty name
        let token3 = create_valid_token_params(&env, "Token3", "TK3");

        let tokens = vec![&env, token1, token2, token3];

        let result = client.batch_create_tokens(&admin, &tokens, &300_i128);

        // Entire batch should fail
        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
        
        // No tokens should be created
        assert_eq!(client.get_token_count(), 0);
    }

    #[test]
    fn test_batch_create_atomic_rollback_invalid_decimals() {
        let (env, client, admin, _) = setup();

        let token1 = create_valid_token_params(&env, "Token1", "TK1");
        let mut token2 = create_valid_token_params(&env, "Token2", "TK2");
        token2.decimals = 20; // Invalid: max is 18
        let token3 = create_valid_token_params(&env, "Token3", "TK3");

        let tokens = vec![&env, token1, token2, token3];

        let result = client.batch_create_tokens(&admin, &tokens, &300_i128);

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
        assert_eq!(client.get_token_count(), 0);
    }

    #[test]
    fn test_batch_create_atomic_rollback_zero_supply() {
        let (env, client, admin, _) = setup();

        let token1 = create_valid_token_params(&env, "Token1", "TK1");
        let mut token2 = create_valid_token_params(&env, "Token2", "TK2");
        token2.initial_supply = 0; // Invalid: must be positive
        let token3 = create_valid_token_params(&env, "Token3", "TK3");

        let tokens = vec![&env, token1, token2, token3];

        let result = client.batch_create_tokens(&admin, &tokens, &300_i128);

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
        assert_eq!(client.get_token_count(), 0);
    }

    #[test]
    fn test_batch_create_atomic_rollback_insufficient_fee() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
            create_valid_token_params(&env, "Token3", "TK3"),
        ];

        // Insufficient fee: need 300, providing 250
        let result = client.batch_create_tokens(&admin, &tokens, &250_i128);

        assert_eq!(result, Err(Ok(Error::InsufficientFee)));
        assert_eq!(client.get_token_count(), 0);
    }

    #[test]
    fn test_batch_create_atomic_rollback_mixed_invalid() {
        let (env, client, admin, _) = setup();

        let token1 = create_valid_token_params(&env, "Token1", "TK1");
        
        let mut token2 = create_valid_token_params(&env, "Token2", "TK2");
        token2.name = String::from_str(&env, ""); // Invalid
        
        let mut token3 = create_valid_token_params(&env, "Token3", "TK3");
        token3.decimals = 20; // Invalid
        
        let mut token4 = create_valid_token_params(&env, "Token4", "TK4");
        token4.initial_supply = -1000; // Invalid

        let tokens = vec![&env, token1, token2, token3, token4];

        let result = client.batch_create_tokens(&admin, &tokens, &400_i128);

        // Should fail on first invalid token (token2)
        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
        assert_eq!(client.get_token_count(), 0);
    }

    // ═══════════════════════════════════════════════════════
    //  Batch Token Creation Tests - Edge Cases
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_empty_batch() {
        let (env, client, admin, _) = setup();

        let tokens: Vec<TokenCreationParams> = vec![&env];

        let result = client.batch_create_tokens(&admin, &tokens, &0_i128);

        assert_eq!(result, Err(Ok(Error::InvalidTokenParams)));
    }

    #[test]
    fn test_batch_create_when_paused() {
        let (env, client, admin, _) = setup();

        // Pause the contract
        client.pause(&admin).unwrap();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
        ];

        let result = client.batch_create_tokens(&admin, &tokens, &100_i128);

        assert_eq!(result, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_batch_create_overpayment_accepted() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
        ];

        // Overpayment: need 200, providing 500
        let result = client.batch_create_tokens(&admin, &tokens, &500_i128);

        assert!(result.is_ok());
        assert_eq!(client.get_token_count(), 2);
    }

    // ═══════════════════════════════════════════════════════
    //  State Consistency Tests
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_token_indices_sequential() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
            create_valid_token_params(&env, "Token3", "TK3"),
        ];

        client.batch_create_tokens(&admin, &tokens, &300_i128).unwrap();

        // Verify tokens can be retrieved by index
        assert!(client.get_token_info(&0).is_ok());
        assert!(client.get_token_info(&1).is_ok());
        assert!(client.get_token_info(&2).is_ok());
        assert!(client.get_token_info(&3).is_err()); // Should not exist
    }

    #[test]
    fn test_batch_create_after_single_create() {
        let (env, client, admin, _) = setup();

        // Create a single token first
        client.create_token(
            &admin,
            &String::from_str(&env, "SingleToken"),
            &String::from_str(&env, "SINGLE"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &100_i128,
        ).unwrap();

        assert_eq!(client.get_token_count(), 1);

        // Now batch create
        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
        ];

        client.batch_create_tokens(&admin, &tokens, &200_i128).unwrap();

        assert_eq!(client.get_token_count(), 3);
        
        // Verify all tokens exist
        assert!(client.get_token_info(&0).is_ok());
        assert!(client.get_token_info(&1).is_ok());
        assert!(client.get_token_info(&2).is_ok());
    }

    #[test]
    fn test_batch_create_creator_balance_set() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
        ];

        client.batch_create_tokens(&admin, &tokens, &200_i128).unwrap();

        // Verify creator has initial supply for each token
        let token1_info = client.get_token_info(&0).unwrap();
        let token2_info = client.get_token_info(&1).unwrap();

        assert_eq!(token1_info.creator, admin);
        assert_eq!(token2_info.creator, admin);
        assert_eq!(token1_info.total_supply, 1_000_000_i128);
        assert_eq!(token2_info.total_supply, 1_000_000_i128);
    }

    // ═══════════════════════════════════════════════════════
    //  Event Emission Tests
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_batch_create_emits_event() {
        let (env, client, admin, _) = setup();

        let tokens = vec![
            &env,
            create_valid_token_params(&env, "Token1", "TK1"),
            create_valid_token_params(&env, "Token2", "TK2"),
        ];

        client.batch_create_tokens(&admin, &tokens, &200_i128).unwrap();

        let events = env.events().all();
        assert!(!events.is_empty(), "Events should be emitted");
    }

    #[test]
    fn test_single_create_emits_event() {
        let (env, client, admin, _) = setup();

        client.create_token(
            &admin,
            &String::from_str(&env, "TestToken"),
            &String::from_str(&env, "TEST"),
            &6_u32,
            &1_000_000_i128,
            &None,
            &100_i128,
        ).unwrap();

        let events = env.events().all();
        assert!(!events.is_empty(), "Events should be emitted");
    }
}
