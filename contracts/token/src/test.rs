#![cfg(test)]
extern crate std;

use crate::{admin::Config, contract::Token, TokenClient};
use soroban_sdk::{
    symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, vec, Address, Env, IntoVal, Symbol
};
use soroban_sdk::TryFromVal;

fn create_token<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    let token = TokenClient::new(e, &e.register_contract(None, Token {}));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let admin2 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);

    let token = create_token(&e, &admin1);

    // Registrar el contrato de MyLab
    let mylab_contract_id = e.register_contract(None, Token {});
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); 

    // Configurar MyLab como el único autorizado
    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone(),
    });

    // 1. MyLab aprueba a user3 para gastar 500 tokens
    token.approve(&user3, &mylab_contract, &500, &200);
    assert_eq!(token.allowance(&user3, &mylab_contract), 500);

    // 2. Mint de 1000 tokens a `mylab_contract` (para que tenga saldo)
    token.mint(&mylab_contract, &1000);
    assert_eq!(token.balance(&mylab_contract), 1000);

    // 3. MyLab transfiere 600 tokens a user1
    token.transfer(&mylab_contract, &user1, &600);
    assert_eq!(token.balance(&mylab_contract), 400);
    assert_eq!(token.balance(&user1), 600);

    // 4. MyLab transfiere 200 tokens a user2
    token.transfer(&mylab_contract, &user2, &200);
    assert_eq!(token.balance(&mylab_contract), 200);
    assert_eq!(token.balance(&user2), 200);

    // 5. MyLab transfiere 100 tokens a user3
    token.transfer(&mylab_contract, &user3, &100);
    assert_eq!(token.balance(&mylab_contract), 100);
    assert_eq!(token.balance(&user3), 100);

    // 6. MyLab transfiere desde user1 a user2 usando `transfer_from`
    token.approve(&user1, &mylab_contract, &500, &200);
    token.transfer_from(&mylab_contract, &user1, &user2, &300);
    assert_eq!(token.balance(&user1), 300);
    assert_eq!(token.balance(&user2), 500);

    // 7. Cambiar administrador
    token.set_admin(&admin2);

    // Verificaciones de autorización
    assert_eq!(
        e.auths(),
        std::vec![
            (
                admin1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("mint"),
                        (&mylab_contract, 1000_i128).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                mylab_contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("approve"),
                        (&user3, &mylab_contract, 500_i128, 200_u32).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                mylab_contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("transfer"),
                        (&mylab_contract, &user1, 600_i128).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                mylab_contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("transfer"),
                        (&mylab_contract, &user2, 200_i128).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                mylab_contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("transfer"),
                        (&mylab_contract, &user3, 100_i128).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                mylab_contract.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        Symbol::new(&e, "transfer_from"),
                        (&mylab_contract, &user1, &user2, 300_i128).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            ),
            (
                admin1.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token.address.clone(),
                        symbol_short!("set_admin"),
                        (&admin2,).into_val(&e),
                    )),
                    sub_invocations: std::vec![]
                }
            )
        ]
    );

    assert_eq!(token.allowance(&mylab_contract, &user3), 500);
    assert_eq!(token.balance(&user1), 300);
    assert_eq!(token.balance(&user2), 500);
    assert_eq!(token.balance(&user3), 100);
}



#[test]
fn test_batch_mint() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let token = create_token(&e, &admin1);
    let mylab_contract_id = e.register_contract(None, Token {}); // Register a test contract
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); // Convert it to Address

    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone()
    });
    // Create Vec<Address> for to_addresses
    let to_addresses = vec![&e, user1.clone(), user2.clone(), user3.clone()];

    // Create Vec<i128> for amounts
    let amounts = vec![&e, 100_i128, 200_i128, 300_i128];
    
    token.batch_mint(
        &to_addresses,
        &amounts,
    );

    assert_eq!(token.balance(&user1), 100);
    assert_eq!(token.balance(&user2), 200);
    assert_eq!(token.balance(&user3), 300);
}

#[test]
fn test_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin);
    let mylab_contract_id = e.register_contract(None, Token {}); // Register a test contract
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); // Convert it to Address

    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone()
    });

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &mylab_contract, &500, &200);
    assert_eq!(token.allowance(&user1, &mylab_contract), 500);

    token.burn_from(&user2, &user1, &500);
    assert_eq!(
        e.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn_from"),
                    (&user2, &user1, 500_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user2), 0);

    token.burn(&user1, &500);
    assert_eq!(
        e.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, 500_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin);
    let mylab_contract_id = e.register_contract(None, Token {}); // Register a test contract
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); // Convert it to Address

    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone()
    });

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

   // token.add_to_whitelist(&vec![&e.clone(), user1.clone()]);
    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn transfer_from_insufficient_allowance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let user3 = Address::generate(&e);
    let token = create_token(&e, &admin);
    let mylab_contract_id = e.register_contract(None, Token {}); // Register a test contract
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); // Convert it to Address

    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone()
    });

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &mylab_contract, &100, &200);
    assert_eq!(token.allowance(&user1, &mylab_contract), 100);

   // token.add_to_whitelist(&vec![&e.clone(), user3.clone()]);

    token.transfer_from(&user3, &user1, &user2, &101);
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.initialize(&admin, &10, &"name".into_val(&e), &"symbol".into_val(&e));
}

#[test]
#[should_panic(expected = "Decimal must not be greater than 18")]
fn decimal_is_over_eighteen() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let token = TokenClient::new(&e, &e.register_contract(None, Token {}));
    token.initialize(&admin, &19, &"name".into_val(&e), &"symbol".into_val(&e));
}

#[test]
fn test_zero_allowance() {
    // Here we test that transfer_from with a 0 amount does not create an empty allowance
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let spender = Address::generate(&e);
    let from = Address::generate(&e);
    let token = create_token(&e, &admin);
    let mylab_contract_id = e.register_contract(None, Token {}); // Register a test contract
    let mylab_contract = Address::try_from_val(&e, &mylab_contract_id.to_val()).unwrap(); // Convert it to Address

    token.set_config(&Config {
        locked_block: 0,
        mylab_contract: mylab_contract.clone()
    });

    //token.add_to_whitelist(&vec![&e.clone(), spender.clone()]);
    token.transfer_from(&spender, &from, &spender, &0);
    assert!(token.get_allowance(&from, &spender).is_none());
}
