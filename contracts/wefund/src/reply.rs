#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, CosmosMsg, BankMsg, QueryRequest, BankQuery, WasmMsg,
    Coin, BalanceResponse, SubMsg, Reply, ContractResult, StdError, from_binary
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse, TokenInfoResponse};

use crate::error::ContractError;
use Interface::wefund::{ExecuteMsg, InstantiateMsg, Config, ProjectState, BackerState,
    VestingParameter,TeamMember, ProjectStatus, Milestone, Vote, WhitelistState
};
use crate::state::{CONFIG, PROJECTSTATES, PROJECT_SEQ, COMMUNITY,  save_projectstate, 
        AUST_AMOUNT, UUSD_AMOUNT, PROJECT_ID, PROFIT};

use crate::contract::{UST};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            let project_id = PROJECT_ID.load(deps.storage)?;
            let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;

            let config = CONFIG.load(deps.storage)?;
            let aust_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
                config.aust_token.clone(),
                &Cw20QueryMsg::Balance{
                    address: _env.contract.address.to_string(),
                }
            )?;

            let amount = aust_balance.balance - AUST_AMOUNT.load(deps.storage)?;
            let last = x.backer_states.len() - 1;
            x.backer_states[last].aust_amount = Coin{
                denom: "aust".to_string(),
                amount: amount
            };
            PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;
            Ok(Response::new()
                .add_attribute("action", "save aust amount")
            )
        },
        2 => {//after release milestone
            //--------Get project info----------------------------
            let project_id = PROJECT_ID.load(deps.storage)?;
            let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;

            //---------get hope to release amount---------------------------
            let step = x.project_milestonestep.u128() as usize;
            let release_amount = 
                x.milestone_states[step].milestone_amount.u128() * UST;
                
            let balance: BalanceResponse = deps.querier.query(
                &QueryRequest::Bank(BankQuery::Balance{
                    address: _env.contract.address.to_string(),
                    denom: "uusd".to_string()
                }
            ))?;
            let withdraw_amount = balance.amount.amount - UUSD_AMOUNT.load(deps.storage)?;

            let ust_release;
            if withdraw_amount > Uint128::from(release_amount) {
                ust_release = Coin::new(release_amount, "uusd");
                
                let mut profit = PROFIT.load(deps.storage)?;
                profit += withdraw_amount - Uint128::from(release_amount);
                PROFIT.save(deps.storage, &profit)?;
            } else {
                ust_release = Coin::new(withdraw_amount.u128(), "uusd");
            }

            let send2_creator = BankMsg::Send { 
                to_address: x.creator_wallet.to_string(),
                amount: vec![ust_release] 
            };

            //--------update project info-------------------
            x.aust_amount -= AUST_AMOUNT.load(deps.storage)?;
            x.backerbacked_amount -= withdraw_amount;
            PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;

            Ok(Response::new()
                .add_message(send2_creator)
                .add_attribute("action", "send2 creator")
            )
        },
        3 => { //after complete project
            //--------Get project info----------------------------
            let project_id = PROJECT_ID.load(deps.storage)?;
            let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;

            //---------get hope to release amount---------------------------
            let balance: BalanceResponse = deps.querier.query(
                &QueryRequest::Bank(BankQuery::Balance{
                    address: _env.contract.address.to_string(),
                    denom: "uusd".to_string()
                }
            ))?;
            let withdraw_amount = balance.amount.amount - UUSD_AMOUNT.load(deps.storage)?;
            let ust_release = Coin::new(withdraw_amount.u128(), "uusd");

            let send2_creator = BankMsg::Send { 
                to_address: x.creator_wallet.to_string(),
                amount: vec![ust_release] 
            };

            //--------update project info-------------------
            x.aust_amount = Uint128::zero();
            x.backerbacked_amount = Uint128::zero();
            x.project_status = ProjectStatus::Done;
            PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;

            Ok(Response::new()
                .add_message(send2_creator)
                .add_attribute("action", "send2 creator")
            )
        },
        4 => {
            let project_id = PROJECT_ID.load(deps.storage)?;
            let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;

            //---------get hope to release amount---------------------------
            let balance: BalanceResponse = deps.querier.query(
                &QueryRequest::Bank(BankQuery::Balance{
                    address: _env.contract.address.to_string(),
                    denom: "uusd".to_string()
                }
            ))?;
            let withdraw_amount = balance.amount.amount - UUSD_AMOUNT.load(deps.storage)?;

            //---------send to backer wallet-------------
            let mut msg = Vec::new();
            for backer in x.backer_states.clone(){
                let mut backed_ust = backer.ust_amount.clone(); 
        
                //---while mistone releasing, suddenly failed, distribute with %
                backed_ust.amount = backer.ust_amount.amount * withdraw_amount
                    /x.backerbacked_amount.clone();
        
                let send2_backer = BankMsg::Send { 
                    to_address: backer.backer_wallet.to_string(),
                    amount: vec![backed_ust] 
                };
                msg.push(CosmosMsg::Bank(send2_backer));
            }
            
            //-----update project state to FAIL----------------------------
            x.aust_amount = Uint128::zero();
            x.backerbacked_amount = Uint128::zero();
            x.project_status = ProjectStatus::Fail;
            PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;

            Ok(Response::new()
            .add_messages(msg)
            .add_attribute("action", "project failed")
            )
        },
        _ => Err(ContractError::InvalidReplyId {}),
    }
}