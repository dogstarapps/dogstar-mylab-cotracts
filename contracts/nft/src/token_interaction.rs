use soroban_sdk::{
    auth::{
        ContractContext, InvokerContractAuthEntry, SubContractInvocation
    }, contract, contractimpl, symbol_short, vec, Address, Env, IntoVal, Symbol, TryIntoVal, Val, Vec
};
use soroban_sdk::Vec as SorobanVec;



/// Private (or public, si prefieres) function to authorize and call
/// `transfer(from, to, amount)` in the Token contract.
fn transfer_as_mylab_internal(
    e: &Env,
    token_contract: Address,
    from: Address,
    to: Address,
    amount: i128,
) {
    // 1) Build a sub-invocation describing: token_contract.transfer(from, to, amount)
    let sub_invocation = SubContractInvocation {
        context: ContractContext {
            contract: token_contract.clone(),
            fn_name: symbol_short!("transfer"),
            args: (from.clone(), to.clone(), amount).into_val(e),
        },
        // If MyLab calls sub-invocations, you'd list them here:
        sub_invocations: vec![e],
    };

    // 2) Convert that sub-invocation into an authorization entry
    let mut auth_entries = SorobanVec::new(e);
    auth_entries.push_back(InvokerContractAuthEntry::Contract(sub_invocation));

    //3) Call authorize_as_current_contract by passing a SorobanVec<InvokerContractAuthEntry>.
    e.authorize_as_current_contract(auth_entries);
    // 4) Finally call `transfer` in the Token contract
    let fn_name = Symbol::new(e, "transfer");
    let args = (from, to, amount).into_val(e);
    let _: () = e.invoke_contract(&token_contract, &fn_name, args);
}

/// Similar function to call `transfer_from(spender, from, to, amount)`
/// with `spender = MyLab`.
fn transfer_from_as_mylab_internal(
    e: &Env,
    token_contract: Address,
    spender: Address,
    from: Address,
    to: Address,
    amount: i128,
) {
    let sub_invocation = SubContractInvocation {
        context: ContractContext {
            contract: token_contract.clone(),
            fn_name: Symbol::new(&e, "transfer_from"),
            args: (spender.clone(), from.clone(), to.clone(), amount).into_val(e),
        },
        sub_invocations: vec![e],
    };

    let mut auth_entries = SorobanVec::new(e);
    auth_entries.push_back(InvokerContractAuthEntry::Contract(sub_invocation));

    e.authorize_as_current_contract(auth_entries);

    let fn_name = Symbol::new(e, "transfer");
    let args = (spender, from, to, amount).into_val(e);
    let _: () = e.invoke_contract(&token_contract, &fn_name, args);
}

/// Similar function to call `approve(from, spender, amount, expiration_ledger)`
/// with `from = MyLab`.
fn approve_as_mylab_internal(
    e: &Env,
    token_contract: Address,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let sub_invocation = SubContractInvocation {
        context: ContractContext {
            contract: token_contract.clone(),
            fn_name: symbol_short!("approve"),
            args: (from.clone(), spender.clone(), amount, expiration_ledger).into_val(e),
        },
        sub_invocations: vec![e],
    };

    let mut auth_entries = SorobanVec::new(e);
    auth_entries.push_back(InvokerContractAuthEntry::Contract(sub_invocation));

    e.authorize_as_current_contract(auth_entries);

    let fn_name = Symbol::new(e, "transfer");
    let args = (from, spender, amount, expiration_ledger).into_val(e);
    let _: () = e.invoke_contract(&token_contract, &fn_name, args);
}
/// Internal function that calls `mint(to, amount)` on the Token contract,
/// acting as MyLab (the admin).
fn call_mint_as_mylab_internal(
    e: &Env,
    token_contract: Address,
    to: Address,
    amount: i128,
) {
    // 1) Build a sub-invocation describing: token_contract.mint(to, amount).
    let sub_invocation = SubContractInvocation {
        context: ContractContext {
            contract: token_contract.clone(),
            fn_name: symbol_short!("mint"),
            args: (to.clone(), amount).into_val(e),
        },
        // If MyLab calls nested sub-invocations, list them here:
        sub_invocations: vec![e],
    };

    // 2) Convert that sub-invocation into an authorization entry in a SorobanVec.
    let mut auth_entries = SorobanVec::new(e);
    auth_entries.push_back(InvokerContractAuthEntry::Contract(sub_invocation));

    // 3) Call authorize_as_current_contract by passing a SorobanVec<InvokerContractAuthEntry>.
    e.authorize_as_current_contract(auth_entries);

    // 4) Finally call `mint` in the token contract.
    //    We assume the token's mint function returns nothing (unit type).
    let fn_name = Symbol::new(e, "mint");
    let args = (to, amount).into_val(e);
    let _: () = e.invoke_contract(&token_contract, &fn_name, args);
}
