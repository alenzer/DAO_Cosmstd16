#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, CosmosMsg, BankMsg, QueryRequest, BankQuery, WasmMsg,
    Coin, AllBalanceResponse, SubMsg, Reply, ContractResult, StdError, from_binary
};

use crate::error::ContractError;
use crate::state::{Config, CONFIG, PROJECTSTATES, ProjectState, BackerState, VestingParameter,
    PROJECT_SEQ, COMMUNITY, Milestone, Vote, save_projectstate, TeamMember, ProjectStatus,
    AUST_AMOUNT};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse, TokenInfoResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            let config = CONFIG.load(deps.storage)?;
            let aust_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
                config.aust_token.clone(),
                &Cw20QueryMsg::Balance{
                    address: _env.contract.address.to_string(),
                }
            )?;
            // let token_addr = Addr::unchecked(res.get_contract_address());

            // register_aterra(deps, token_addr)
            Ok(Response::new())
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}