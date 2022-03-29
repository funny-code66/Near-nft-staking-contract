use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::TokenId;
use near_contract_standards::non_fungible_token::Token;
use near_primitives::views::FinalExecutionStatus;
use near_units::parse_near;
use near_sdk::json_types::U128;
use near_sdk::ONE_YOCTO;
use workspaces::prelude::DevAccountDeployer;
use workspaces::{Account, Contract, DevNetwork, Worker};
extern crate cross_contract_high_level;

pub const TOKEN_ID: &str = "0";

/// # Note
/// 
/// In workspace-rs, a user cannot pass caller account explicitly.
/// Passing certain caller to test function is important 
/// in NFTxxx.transfer_from() and NFTxxx.approve().
/// 
/// # TODO
/// 
/// Make test code using near_sdk_sim.


pub async fn init(
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<(Contract, Contract, Contract, UserAccount, Account, Account)> {
    let nft_contract =
        worker.dev_deploy(include_bytes!("../../non-fungible-token/res/non_fungible_token.wasm").to_vec()).await?;
    println!("***************************************************** 1");

    let ft_contract =
        worker.dev_deploy(include_bytes!("../../fungible-token/res/fungible_token.wasm").to_vec()).await?;

    let staking_contract = worker.dev_deploy(include_bytes!("../res/cross_contract_high_level.wasm").to_vec()).await?;
    println!("***************************************************** 2");

    let res = nft_contract
        .call(&worker, "new_default_meta")
        .args_json((nft_contract.id(),))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
        
    println!("***************************************************** 3");

    let res = nft_contract
        .as_account()
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("9 N"))
        .transact()
        .await?;
    assert!(matches!(res.details.status, FinalExecutionStatus::SuccessValue(_)));
    let alice = res.result;

    let res = nft_contract
        .as_account()
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("9 N"))
        .transact()
        .await?;
    assert!(matches!(res.details.status, FinalExecutionStatus::SuccessValue(_)));
    let bob = res.result;

    println!("***************************************************** 4");

    let mut genesis = near_sdk_sim::runtime::GenesisConfig::default();
    genesis.gas_limit = u64::MAX;
    genesis.gas_price = 0;
    let master_account = init_simulator(Some(genesis));
    println!("***************************************************** 5");
    return Ok((staking_contract, nft_contract, ft_contract, master_account, alice, bob));
}

#[tokio::test]
async fn test_nft() -> anyhow::Result<()>  {
    let worker = workspaces::sandbox();
    let initial_balance = U128::from(parse_near!("9 N"));
    let (staking_contract, nft_contract, ft_contract, master_account, alice, _) = init(&worker).await?;
    
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((alice.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    println!("***************************************************** 6");
    assert_eq!(owner_tokens.len(), 0);
    
    
    let token_metadata = TokenMetadata {
        title: Some("Olympus Mons".into()),
        description: Some("The tallest mountain in the charted solar system".into()),
        media: None,
        media_hash: None,
        copies: Some(1u64),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };

    let res = nft_contract
        .call(&worker, "nft_mint")
        .args_json((TOKEN_ID, nft_contract.id(), token_metadata))?
        .gas(300_000_000_000_000)
        .deposit(parse_near!("7 mN"))
        .transact()
        .await?;
    println!("***************************************************** 7");
    
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));

    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((nft_contract.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    println!("***************************************************** 8");
    
    assert_eq!(owner_tokens.len(), 1);

    assert_eq!(owner_tokens.get(0).unwrap().token_id, "0".to_string());

    let res = ft_contract
        .call(&worker, "new_default_meta")
        .args_json((alice.id(), initial_balance))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("***************************************************** 9");

    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    

    let res = ft_contract.call(&worker, "ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, initial_balance);

    let root_balance = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json((nft_contract.id(),))?
        .view()
        .await?
        .json::<U128>()?;
    println!("***************************************************** 10");

    assert_eq!(root_balance, U128::from(parse_near!("0 N")));
    
    let res = nft_contract
        .call(&worker, "nft_approve")
        .args_json((TOKEN_ID, staking_contract.id(), Option::<String>::None))?
        .gas(300_000_000_000_000)
        .deposit(510000000000000000000)
        .transact()
        .await?;
    println!("***************************************************** 11");
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    let res = nft_contract
        .call(&worker, "nft_transfer")
        .args_json((
            alice.id(),
            TOKEN_ID,
            Option::<u64>::None,
            Some("simple transfer".to_string()),
        ))?
        .gas(300_000_000_000_000)
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(matches!(res.status, FinalExecutionStatus::SuccessValue(_)));
    let owner_tokens: Vec<Token> = nft_contract
        .call(&worker, "nft_tokens_for_owner")
        .args_json((alice.id(), Option::<U128>::None, Option::<u64>::None))?
        .view()
        .await?
        .json()?;
    println!("***************************************************** 12");
    assert_eq!(owner_tokens.len(), 1);
    
    Ok(()) 
}
