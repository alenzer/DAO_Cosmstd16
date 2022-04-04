#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, to_binary, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, CosmosMsg, BankMsg, QueryRequest, BankQuery, WasmMsg,
    Coin, BalanceResponse, SubMsg, AllBalanceResponse
};
use cw2::set_contract_version;
use cw_storage_plus::{U128Key};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse, TokenInfoResponse};

use crate::error::ContractError;
use Interface::wefund::{ExecuteMsg, InstantiateMsg, Config, ProjectState, BackerState,
    VestingParameter,TeamMember, ProjectStatus, Milestone, Vote, WhitelistState
};
use crate::state::{CONFIG, PROJECTSTATES, PROJECT_SEQ, COMMUNITY,  save_projectstate, 
        AUST_AMOUNT, UUSD_AMOUNT, PROJECT_ID, PROFIT};

use Interface::market::{ExecuteMsg as AnchorMarket, Cw20HookMsg,
    QueryMsg as AnchorQuery, EpochStateResponse};                    
use Interface::vesting::{ExecuteMsg as VestingMsg, QueryMsg as VestingQuery,
    VestingParameter as VestingParam, 
    ProjectInfo as VestingProjectInfo};
use Interface::staking::{CardType};

// version info for migration info
const CONTRACT_NAME: &str = "WEFUND";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const UST: u128 = 1_000_000; //ust unit

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .admin
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(info.sender.clone());

    let wefund = msg
        .wefund
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(info.sender.clone());

    let anchor_market = msg
        .anchor_market
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(Addr::unchecked(
            // String::from("terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s")));//main net
            String::from("terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal")));//test net
    let aust_token = msg
        .aust_token
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(Addr::unchecked(
            // String::from("terra1hzh9vpxhsk8253se0vv5jj6etdvxu3nv8z07zu")));//main net
            String::from("terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl")));//test net

    let vesting_contract = msg
        .vesting_contract
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(Addr::unchecked("".to_string()));

    let config = Config {
        owner, wefund, anchor_market, aust_token, vesting_contract
    };

    CONFIG.save(deps.storage, &config)?;
    PROJECT_SEQ.save(deps.storage, &Uint128::zero())?;
    COMMUNITY.save(deps.storage, &Vec::new())?;

    AUST_AMOUNT.save(deps.storage, &Uint128::zero())?;
    UUSD_AMOUNT.save(deps.storage, &Uint128::zero())?;
    PROJECT_ID.save(deps.storage, &Uint128::zero())?;

    PROFIT.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetConfig{ admin, wefund, anchor_market, aust_token, vesting_contract } 
            => try_setconfig(deps, _env, info, admin, wefund, anchor_market, aust_token, vesting_contract),
        ExecuteMsg::AddProject { 
            project_id,
            project_company,
            project_title,
            project_description,
            project_ecosystem,
            project_createddate,
            project_saft,
            project_logo,
            project_whitepaper,
            project_website,
            project_email,
            creator_wallet,
            project_collected,
            project_milestones,
            project_teammembers,
            vesting,
            token_addr,

            country,
            cofounder_name,
            service_wefund,
            service_charity,
            professional_link,
        } => 
            try_addproject(deps, _env, info, 
                project_id,
                project_company,
                project_title,
                project_description,
                project_ecosystem,
                project_createddate,
                project_saft,
                project_logo,
                project_whitepaper,
                project_website,
                project_email,
                creator_wallet,
                project_collected,
                project_milestones,
                project_teammembers,
                vesting,
                token_addr,

                country,
                cofounder_name,
                service_wefund,
                service_charity,
                professional_link,
            ),

        ExecuteMsg::Back2Project { project_id, backer_wallet, fundraising_stage, token_amount, otherchain, otherchain_wallet} => 
            try_back2project(deps, _env, info, project_id, backer_wallet, fundraising_stage, token_amount, otherchain, otherchain_wallet),

        ExecuteMsg::CompleteProject{ project_id } =>
            try_completeproject(deps, _env, project_id ),

        ExecuteMsg::FailProject{ project_id } =>
            try_failproject(deps, _env, project_id),
        
        ExecuteMsg::RemoveProject{ project_id } =>
            try_removeproject(deps, info, project_id),
        
        ExecuteMsg::TransferAllCoins{wallet} =>
            try_transferallcoins(deps, _env, info, wallet),

        ExecuteMsg::AddCommunitymember{wallet} =>
            try_addcommunitymember(deps, wallet),

        ExecuteMsg::RemoveCommunitymember{wallet} =>
            try_removecommunitymember(deps, wallet),

        ExecuteMsg::WefundApprove{project_id} =>
            try_wefundapprove(deps, info, project_id),

        ExecuteMsg::SetFundraisingStage{project_id, stage} =>
            try_setfundraisingstage(deps, project_id, stage),
        
        ExecuteMsg::SetMilestoneVote{project_id, wallet, voted} =>
            try_setmilestonevote(deps, _env, info, project_id, wallet, voted),

        ExecuteMsg::ReleaseMilestone{project_id} =>
            try_releasemilestone(deps, _env, project_id),

        ExecuteMsg::SetProjectStatus{project_id, status} =>
            try_setprojectstatus(deps, info, project_id, status),

        ExecuteMsg::OpenWhitelist{project_id, holder_alloc} =>
            try_openwhitelist(deps, _env, info, project_id, holder_alloc),
        
        ExecuteMsg::RegisterWhitelist{project_id, card_type} =>
            try_registerwhitelist(deps, _env, info, project_id, card_type),

        ExecuteMsg::CloseWhitelist{project_id} =>
            try_closewhitelist(deps, _env, info, project_id)
    }
}
pub fn try_setprojectstatus(deps: DepsMut, info: MessageInfo, project_id: Uint128, status: Uint128)
    ->Result<Response, ContractError>
{
    //-----------check owner--------------------------
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }
//    let x:ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u128().into())?;
    //-------update-------------------------
    PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
        None => Err(ContractError::NotRegisteredProject {}),
        Some(mut project) => {
            if status == Uint128::zero() {
                project.project_status = ProjectStatus::WefundVote;
            } else if status == Uint128::from(1u64){
                project.project_status = ProjectStatus::Whitelist;
            } else if status == Uint128::from(2u64){
                project.project_status = ProjectStatus::Fundraising;
            } else if status == Uint128::from(3u64){
                project.project_status = ProjectStatus::Releasing;
            } else if status == Uint128::from(4u64){
                project.project_status = ProjectStatus::Done;
            } else if status == Uint128::from(5u64){
                project.project_status = ProjectStatus::Fail;
            }
            Ok(project)
        }
    })?;
    Ok(Response::new()
    .add_attribute("action", "Set project status"))
}
pub fn convert_str_int(str: String)
    ->u128
{
    let bytes = str.into_bytes();
    let mut res: u128 = 0;
    let mut dot = false;
    let mut dotbelow = 0;

    for i in 0..bytes.len(){
        if bytes[i] < 48{
            dot = true;
        }
        else if dotbelow < 6 {
            res = res * 10 + (bytes[i] - 48) as u128;
            if dot {
                dotbelow += 1;
            }
        }
    }
    return res;
}
pub fn try_releasemilestone(deps: DepsMut, _env: Env, _project_id: Uint128 ) 
    -> Result<Response, ContractError>
{
    //--------Get project info----------------------------
    let x:ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u128().into())?;

    //--------Checking project status-------------------------
    if x.project_status != ProjectStatus::Releasing{//only releasing status
        return Err(ContractError::NotCorrectStatus{status: x.project_status as u32});
    }

    //---------get hope to release amount---------------------------
    let step = x.project_milestonestep.u128() as usize;
    let release_amount = 
        x.milestone_states[step].milestone_amount.u128() * UST;

    //---------calc total deposited to project----------------
    let total_deposited = x.backerbacked_amount.u128();
    let withdraw_amount = release_amount * x.aust_amount.u128() / total_deposited;

    //----------load config and read aust token address-----------------
    let config = CONFIG.load(deps.storage).unwrap();
    //----ask aust_token for transfer to anchor martket and execute redeem_stable ----------
    let withdraw = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(config.aust_token),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: config.anchor_market.to_string(),
                msg: to_binary(&Cw20HookMsg::RedeemStable{}).unwrap(), //redeem_stable{}
                amount: Uint128::new(withdraw_amount)
            }).unwrap(),
            funds: Vec::new()
        }),
        2
    );

    let balance: BalanceResponse = deps.querier.query(
        &QueryRequest::Bank(BankQuery::Balance{
            address: _env.contract.address.to_string(),
            denom: "uusd".to_string()
        }
    ))?;
    AUST_AMOUNT.save(deps.storage, &Uint128::from(withdraw_amount))?;
    UUSD_AMOUNT.save(deps.storage, &balance.amount.amount)?;
    PROJECT_ID.save(deps.storage, &_project_id)?;

    Ok(Response::new()
    .add_submessage(withdraw)
    .add_attribute("action", "release milestone")
    .add_attribute("withdraw aust amount", withdraw_amount.to_string())
    .add_attribute("total_deposited", total_deposited.to_string())
    )
}
pub fn try_setmilestonevote(deps: DepsMut, _env:Env, info:MessageInfo, project_id: Uint128, wallet: String, voted: bool)
    -> Result<Response, ContractError>
{
    let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    
    //-------check project status-------------------
    if x.project_status != ProjectStatus::Releasing { //only releasing status
        return Err(ContractError::NotCorrectStatus{status:x.project_status as u32});
    }

    let wallet = deps.api.addr_validate(&wallet).unwrap();
    let step = x.project_milestonestep.u128() as usize;

    if x.milestone_states[step].milestone_status != Uint128::zero(){//only voting status
        return Err(ContractError::NotCorrectMilestoneStatus{
            step:step, status:x.milestone_states[step].milestone_status 
        })
    }

    //------set vot status and check all voted for same backer--------------------
    // let index = x.milestone_states[step].milestone_votes.iter().position(|x|x.wallet == wallet).unwrap();
    // x.milestone_states[step].milestone_votes[index].voted = voted;

    let mut all_voted = true;
    for vote in x.milestone_states[step].milestone_votes.iter_mut() {
        if vote.wallet == wallet{
            vote.voted = voted;
        }
        all_voted = all_voted & vote.voted;
    }

    let mut deps = deps;
    if all_voted{
        x.milestone_states[step].milestone_status = Uint128::new(1); //switch to releasing status
        //-----------------release function---------------
        let res = execute(deps.branch(), _env, info, 
                    ExecuteMsg::ReleaseMilestone{project_id});

        x.milestone_states[step].milestone_status = Uint128::new(2); //switch to released status
        x.project_milestonestep += Uint128::new(1); //switch to next milestone step
        
        //-----------check milestone done---------------------
        if x.project_milestonestep >= Uint128::new(x.milestone_states.len() as u128){
            x.project_status = ProjectStatus::Done; //switch to project done status
        }

        //-------update-------------------------
        PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
            None => Err(ContractError::NotRegisteredProject {}),
            Some(mut project) => {
                project.milestone_states = x.milestone_states;
                project.project_milestonestep = x.project_milestonestep;
                project.project_status = x.project_status;
                Ok(project)
            }
        })?;
        return res;
    }
    //-------update-------------------------
    PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
        None => Err(ContractError::NotRegisteredProject {}),
        Some(mut project) => {
            project.milestone_states = x.milestone_states;
            project.project_milestonestep = x.project_milestonestep;
            project.project_status = x.project_status;
            Ok(project)
        }
    })?;

    Ok(Response::new()
    .add_attribute("action", "Set milestone vote")
    )
}

pub fn try_setfundraisingstage(deps: DepsMut, project_id: Uint128, stage: Uint128)
    -> Result<Response, ContractError>
{
    PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
        None => Err(ContractError::NotRegisteredProject {}),
        Some(mut project) => {
            project.fundraising_stage = stage;
            Ok(project)
        }
    })?;

    Ok(Response::new()
    .add_attribute("action", "Set Fundraising stage")
    )
}

pub fn try_wefundapprove(deps: DepsMut, info:MessageInfo, project_id: Uint128)
    ->Result<Response, ContractError>
{
    //-----------check owner--------------------------
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.owner{
        return Err(ContractError::Unauthorized{});
    }

    let mut x:ProjectState = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    
    //-------check project status-------------------
    if x.project_status != ProjectStatus::WefundVote { //only wefund approve status
        return Err(ContractError::NotCorrectStatus{status:x.project_status as u32});
    }
    x.project_status = ProjectStatus::Fundraising; //switch to fundraising status

    PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
        None => Err(ContractError::NotRegisteredProject {}),
        Some(mut project) => {
            project.project_status = x.project_status;
            Ok(project)
        }
    })?;

    Ok(Response::new()
    .add_attribute("action", "Wefund Approve")
    )
}

pub fn try_removecommunitymember(deps:DepsMut, wallet: String)
    -> Result<Response, ContractError>
{
    let wallet = deps.api.addr_validate(&wallet).unwrap();

    let mut community = COMMUNITY.load(deps.storage).unwrap();
    let res = community.iter().find(|&x| x == &wallet);
    if res == None {
        return Err(ContractError::NotRegisteredCommunity{});
    }

    community.retain(|x|x != &wallet);
    COMMUNITY.save(deps.storage, &community)?;

    Ok(Response::new()
    .add_attribute("action", "remove community member")
    )
}

pub fn try_addcommunitymember(deps:DepsMut, wallet: String)
    -> Result<Response, ContractError>
{
    let wallet = deps.api.addr_validate(&wallet).unwrap();

    let mut community = COMMUNITY.load(deps.storage).unwrap();
    let res = community.iter().find(|&x| x == &wallet);
    if res != None {
        return Err(ContractError::AlreadyRegisteredCommunity{});
    }

    community.push(wallet);
    COMMUNITY.save(deps.storage, &community)?;

    Ok(Response::new()
    .add_attribute("action", "add community member")
    )
}
pub fn try_transferallcoins(deps:DepsMut, _env:Env, info:MessageInfo, wallet:String)
    -> Result<Response, ContractError>
{
    //-----------check owner--------------------------
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.owner{
        return Err(ContractError::Unauthorized{});
    }
    //--------get all native coins and ust - 4 ----------------------
    let balance: AllBalanceResponse = deps.querier.query(
        &QueryRequest::Bank(BankQuery::AllBalances {
            address: _env.contract.address.to_string(),
        }
    ))?;

    let mut nativecoins:Vec<Coin> = Vec::new();
    for mut x in balance.amount
    {
        if x.denom == "uusd" {
            if x.amount.u128() < 4000000 {
                return Err(ContractError::NeedCoin{});
            }
            x.amount = Uint128::new(x.amount.u128() - 4000000);
        }
        nativecoins.push(x);
    }

    let bank_native = BankMsg::Send { 
        to_address: wallet.clone(),
        amount: nativecoins,
    };

    //--------transfer all aust--------------------------------
    let config = CONFIG.load(deps.storage).unwrap();

    let aust_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
        config.aust_token.clone(),
        &Cw20QueryMsg::Balance{
            address: _env.contract.address.to_string(),
        }
    )?;
    let bank_aust = WasmMsg::Execute {
        contract_addr: String::from(config.aust_token),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: wallet,
            amount: aust_balance.balance,
        }).unwrap(),
        funds: Vec::new()
    };

    Ok(Response::new()
    .add_messages(vec![
        CosmosMsg::Bank(bank_native),
        CosmosMsg::Wasm(bank_aust)])
    .add_attribute("action", "trasnfer all coins")
    )
}
pub fn try_removeproject(deps:DepsMut, info:MessageInfo, project_id:Uint128)
    -> Result<Response, ContractError>
{
    //-----------check owner--------------------------
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.owner
    {
        return Err(ContractError::Unauthorized{});
    }

    return remove_project(deps, project_id);
}
pub fn remove_project(deps:DepsMut, _project_id:Uint128)
    ->Result<Response, ContractError>
{
    let res = PROJECTSTATES.may_load(deps.storage, _project_id.u128().into());
    if res == Ok(None) {
        return Err(ContractError::NotRegisteredProject {});
    }
    PROJECTSTATES.remove(deps.storage, U128Key::new(_project_id.u128()));
    Ok(Response::new())
}
pub fn try_setconfig(deps:DepsMut, _env:Env, info:MessageInfo,
    admin: Option<String>, 
    wefund: Option<String>, 
    anchor_market: Option<String>, 
    aust_token: Option<String>,
    vesting_contract: Option<String>
) -> Result<Response, ContractError>
{
    //-----------check owner--------------------------
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }
    
    let mut config = CONFIG.load(deps.storage).unwrap();

    config.owner =  admin
    .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
    .unwrap_or(config.owner);

    config.wefund = wefund
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(config.wefund);

    config.anchor_market = anchor_market
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(config.anchor_market);

    config.aust_token = aust_token
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(config.aust_token);

    config.vesting_contract = vesting_contract
        .and_then(|s| deps.api.addr_validate(s.as_str()).ok()) 
        .unwrap_or(config.vesting_contract);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "SetConfig"))                                
}
pub fn try_completeproject(
    deps: DepsMut,
    _env: Env,
    _project_id: Uint128
) -> Result<Response, ContractError>
{
    //--------Get project info----------------------------
    let x:ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u128().into())?;

    //--------Checking project status-------------------------
    if x.project_status != ProjectStatus::Releasing{//only releasing status
        return Err(ContractError::NotCorrectStatus{status: x.project_status as u32});
    }

    let withdraw_amount = x.aust_amount.u128();

    //----------load config and read aust token address-----------------
    let config = CONFIG.load(deps.storage).unwrap();

    //----ask aust_token for transfer to anchor martket and execute redeem_stable ----------
    let withdraw = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(config.aust_token),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: config.anchor_market.to_string(),
                msg: to_binary(&Cw20HookMsg::RedeemStable{}).unwrap(), //redeem_stable{}
                amount: Uint128::new(withdraw_amount)
            }).unwrap(),
            funds: Vec::new()
        }),
        3
    );

    let balance: BalanceResponse = deps.querier.query(
        &QueryRequest::Bank(BankQuery::Balance{
            address: _env.contract.address.to_string(),
            denom: "uusd".to_string()
        }
    ))?;
    UUSD_AMOUNT.save(deps.storage, &balance.amount.amount)?;
    PROJECT_ID.save(deps.storage, &_project_id)?;

    Ok(Response::new()
    .add_submessage(withdraw)
    .add_attribute("action", "complete milestone")
    .add_attribute("withdraw aust amount", withdraw_amount.to_string())
    )
}
pub fn try_failproject(
    deps: DepsMut,
    _env: Env,
    _project_id: Uint128
) -> Result<Response, ContractError>
{
    //--------Get project info----------------------------
    let x:ProjectState = PROJECTSTATES.load(deps.storage, _project_id.u128().into())?;

    //--------Checking project status-------------------------
    if x.project_status != ProjectStatus::Releasing{//only releasing status
        return Err(ContractError::NotCorrectStatus{status: x.project_status as u32});
    }

    let withdraw_amount = x.aust_amount.u128();

    //----------load config and read aust token address-----------------
    let config = CONFIG.load(deps.storage).unwrap();

    //----ask aust_token for transfer to anchor martket and execute redeem_stable ----------
    let withdraw = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(config.aust_token),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: config.anchor_market.to_string(),
                msg: to_binary(&Cw20HookMsg::RedeemStable{}).unwrap(), //redeem_stable{}
                amount: Uint128::new(withdraw_amount)
            }).unwrap(),
            funds: Vec::new()
        }),
        4
    );

    let balance: BalanceResponse = deps.querier.query(
        &QueryRequest::Bank(BankQuery::Balance{
            address: _env.contract.address.to_string(),
            denom: "uusd".to_string()
        }
    ))?;
    UUSD_AMOUNT.save(deps.storage, &balance.amount.amount)?;
    PROJECT_ID.save(deps.storage, &_project_id)?;

    Ok(Response::new()
    .add_submessage(withdraw)
    .add_attribute("action", "failed milestone")
    .add_attribute("withdraw aust amount", withdraw_amount.to_string())
    )
}

pub fn try_addproject(
    deps:DepsMut,
    _env: Env,
    _info: MessageInfo,
    _project_id: Uint128,
    _project_company: String,
    _project_title: String,
    _project_description: String,
    _project_ecosystem: String,
    _project_createddate: String,
    _project_saft: String,
    _project_logo: String,
    _project_whitepaper: String,
    _project_website: String,
    _project_email: String,
    _creator_wallet: String,
    _project_collected: Uint128,
    _project_milestones: Vec<Milestone>,
    _project_teammembers: Vec<TeamMember>,
    _vesting: Vec<VestingParameter>,
    _token_addr: String,

    _country: String,
    _cofounder_name: String,
    _service_wefund: String,
    _service_charity: String,
    _professional_link: String,
) -> Result<Response, ContractError> 
{
    let token_addr = deps.api.addr_validate(_token_addr.as_str())
        .unwrap_or(Addr::unchecked("".to_string()));

    let mut new_project:ProjectState = ProjectState{
        project_company: _project_company,
        project_title: _project_title,
        project_description: _project_description,
        project_ecosystem: _project_ecosystem,
        project_createddate: _project_createddate,
        project_saft: _project_saft,
        project_logo: _project_logo,
        project_whitepaper: _project_whitepaper,
        project_website: _project_website,
        project_email: _project_email,
//-----------------------------------
        project_id: Uint128::zero(), //auto increment
        project_status: ProjectStatus::WefundVote,
        fundraising_stage: Uint128::zero(),

        backerbacked_amount: Uint128::zero(),
        aust_amount: Uint128::zero(),

        backer_states: Vec::new(),

        project_milestonestep: Uint128::zero(), //first milestonestep

        whitelist: Vec::new(),
        holder_alloc: Uint128::zero(),
        holder_ticket: Uint128::zero(),
        community_ticket: Uint128::zero(),
//-------------------------------------------
        creator_wallet: deps.api.addr_validate(&_creator_wallet).unwrap(),
        project_collected: _project_collected,

        milestone_states: _project_milestones,
        teammember_states: _project_teammembers,
        vesting: _vesting.clone(),
        token_addr: token_addr.clone(),
//---------------------------------------------------
        country: _country,
        cofounder_name: _cofounder_name,
        service_wefund: _service_wefund,
        service_charity: _service_charity,
        professional_link: _professional_link 
    };

    if _project_id == Uint128::zero() {
        save_projectstate(deps.storage, &mut new_project)?;
    }else{
        let x = PROJECTSTATES.load(deps.storage, _project_id.u128().into())?;
        new_project.project_id = x.project_id;
        new_project.project_status = x.project_status;
        new_project.fundraising_stage = x.fundraising_stage;
        new_project.backerbacked_amount = x.backerbacked_amount;
        new_project.aust_amount = x.aust_amount;
        new_project.backer_states = x.backer_states;
        new_project.project_milestonestep = x.project_milestonestep;
        new_project.whitelist = x.whitelist;
        new_project.holder_alloc = x.holder_alloc;
        new_project.holder_ticket = x.holder_ticket;
        new_project.community_ticket = x.community_ticket;
        PROJECTSTATES.save(deps.storage, _project_id.u128().into(), &new_project)?;
    }

    let config = CONFIG.load(deps.storage)?;
    if config.vesting_contract != "".to_string() && token_addr != "".to_string()
    {
        let mut vesting_params: Vec<VestingParam> = Vec::new();
        for param in _vesting{
            vesting_params.push(VestingParam{
                soon: param.stage_soon,
                after: param.stage_after,
                period: param.stage_period,
            })
        }
        //----------add fundraising project------------------------
        let add_vesting_project = WasmMsg::Execute {
            contract_addr: config.vesting_contract.to_string(),
            msg: to_binary(
                &VestingMsg::AddProject {
                    project_id: new_project.project_id,
                    admin: _env.contract.address.to_string(),
                    token_addr: token_addr.to_string(),
                    vesting_params: vesting_params,
                    start_time: Uint128::zero()
                }
            ).unwrap(),
            funds: vec![]
        };

        return Ok(Response::new()
            .add_messages(vec![CosmosMsg::Wasm(add_vesting_project)])
            .add_attribute("action", "add project")
            .add_attribute("id", new_project.project_id)
            );
    }

    Ok(Response::new()
        .add_attribute("action", "add project"))
}

pub fn try_back2project(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    project_id: Uint128, 
    backer_wallet: String,
    fundraising_stage: Uint128,
    token_amount: Uint128,
    otherchain: String,
    otherchain_wallet: String,
) -> Result<Response, ContractError> 
{
    //-------check project exist-----------------------------------
    let res = PROJECTSTATES.may_load(deps.storage, project_id.u128().into());
    if res == Ok(None) { //not exist
        return Err(ContractError::NotRegisteredProject {});
    }
    //--------Get project info------------------------------------
    let mut x = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    if x.project_status != ProjectStatus::Fundraising{//only fundraising status
        return Err(ContractError::NotCorrectStatus{status: x.project_status as u32});
    }

    //--------check sufficient back--------------------
    let fee: u128 = 4 * UST;
    if info.funds.is_empty() || info.funds[0].amount.u128() < 6 * UST{
        return Err(ContractError::NeedCoin{});
    }
 
    let fund = info.funds[0].clone();
    let mut fund_real_back = fund.clone();
    let mut fund_wefund = fund.clone();
    //--------calc amount to desposit and to wefund
    if fund.amount.u128() >= 100 * UST{
        fund_real_back.amount = Uint128::new(fund.amount.u128() * 100 / 105);
        fund_wefund.amount = Uint128::new((fund.amount.u128() * 5 / 105) - fee);
    } else {
        fund_real_back.amount = Uint128::new(fund.amount.u128() - 5 * UST);
        fund_wefund.amount = Uint128::new(1 * UST);
    }

    let backer_wallet = deps.api.addr_validate(&backer_wallet)?;

    //--------check backed amount----------------
    let collected = x.project_collected;

    // if x.backerbacked_amount >= collected{
    //     return Err(ContractError::AlreadyCollected{});
    // }
    //-----sum in whitelist-------------------
    let index = x.whitelist.iter().position(|x| x.wallet == backer_wallet.clone());
    if index == None{
        return Err(ContractError::NotRegisteredWhitelist{ });
    }

    x.whitelist[index.unwrap()].backed += fund_real_back.amount;
    x.backerbacked_amount += fund_real_back.amount;

    let new_baker:BackerState = BackerState{
        backer_wallet: backer_wallet.clone(),
        otherchain: otherchain,
        otherchain_wallet: otherchain_wallet,
        ust_amount: fund_real_back.clone(),
        aust_amount: Coin::new(0, "aust")
    };

    x.backer_states.push(new_baker);

    //------check needback-----------------
    let mut backer_needback = true;

    if x.backerbacked_amount  >= collected{
        backer_needback = false;
    }

    let mut msgs: Vec<CosmosMsg> = vec![];

    //---------check collection and switch to releasing status---------
    if backer_needback == false{
        x.project_status = ProjectStatus::Releasing; //releasing

        //------add milestone votes in every milestone---------------
        let mut milestone_votes = Vec::new();
        for backer in x.backer_states.clone(){
            milestone_votes.push(
                Vote{ wallet: backer.backer_wallet, voted: false }
            );
        }
        //-----add wefund vote------------------
        let config = CONFIG.load(deps.storage)?;
        milestone_votes.push(
            Vote{ wallet: config.owner, voted: true}
        );

        for i in 0..(x.milestone_states.len() as usize){
            x.milestone_states[i].milestone_votes = milestone_votes.clone();
        }

        if config.vesting_contract != "".to_string() && x.token_addr != "".to_string() {
            let vesting = x.vesting.clone();
            let mut token_amount = Uint128::zero();
            for stage in vesting {
                token_amount += stage.stage_amount;
            }

            let token_info: TokenInfoResponse = deps.querier.query_wasm_smart(
                x.token_addr.clone(),
                &Cw20QueryMsg::TokenInfo{}
            )?;

            token_amount = token_amount * 
                Uint128::new((10 as u128).pow(token_info.decimals as u32)); //for decimals
            
            let token_transfer = WasmMsg::Execute {
                contract_addr: x.token_addr.to_string(),
                msg: to_binary(
                    &Cw20ExecuteMsg::TransferFrom{
                        owner: x.creator_wallet.to_string(),
                        recipient: config.vesting_contract.to_string(),
                        amount: token_amount
                    }
                ).unwrap(),
                funds: vec![]
            };
            msgs.push(CosmosMsg::Wasm(token_transfer));
            //---------start vesting-----------------------------
            let start_vesting = WasmMsg::Execute {
                contract_addr: config.vesting_contract.to_string(),
                msg: to_binary(
                    &VestingMsg::StartRelease {
                        project_id: x.project_id,
                        start_time: Uint128::from(env.block.time.seconds()),
                    }
                ).unwrap(),
                funds: vec![]
            };
            msgs.push(CosmosMsg::Wasm(start_vesting));
        }
    }

    PROJECTSTATES.update(deps.storage, project_id.u128().into(), |op| match op {
        None => Err(ContractError::NotRegisteredProject {}),
        Some(mut project) => {
            project.project_status = x.project_status.clone();
            project.backerbacked_amount = x.backerbacked_amount;
            project.backer_states = x.backer_states;
            
            if x.project_status == ProjectStatus::Releasing{//only on switching releasing status
                project.milestone_states = x.milestone_states;
            }
            Ok(project)
        }
    })?;

    //----------load config and read anchor market address-----------------
    let config = CONFIG.load(deps.storage).unwrap();
    let anchormarket = config.anchor_market;

    //----------deposite to anchor market------------------------
    let deposit_msg: SubMsg = SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(anchormarket),
            msg: to_binary(&AnchorMarket::DepositStable {}).unwrap(),
            funds: vec![fund_real_back]
        }),
        1
    );

    //---------send to Wefund with 5/105--------------------
    let bank_wefund = BankMsg::Send { 
        to_address: config.wefund.to_string(),
        amount: vec![fund_wefund] 
    };
    msgs.push(CosmosMsg::Bank(bank_wefund));

    if config.vesting_contract != "".to_string(){
        //----------add fundraising user------------------------
        let add_fundraising_user = WasmMsg::Execute {
            contract_addr: config.vesting_contract.to_string(),
            msg: to_binary(
                &VestingMsg::AddUser {
                    project_id: project_id,
                    wallet: info.sender,
                    stage: fundraising_stage,
                    amount: token_amount,
                }
            ).unwrap(),
            funds: vec![]
        };
        msgs.push(CosmosMsg::Wasm(add_fundraising_user));
    }

    let aust_balance: Cw20BalanceResponse = deps.querier.query_wasm_smart(
        config.aust_token,
        &Cw20QueryMsg::Balance{
            address: env.contract.address.to_string(),
        }
    )?;

    AUST_AMOUNT.save(deps.storage, &aust_balance.balance)?;
    PROJECT_ID.save(deps.storage, &project_id)?;

    Ok(Response::new()
    .add_submessage(deposit_msg)
    .add_messages(msgs)
    .add_attribute("action", "back to project")
    )
}

pub fn try_openwhitelist(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    project_id: Uint128,
    holder_alloc: Uint128
) 
    -> Result<Response, ContractError> 
{
    let mut x = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    if info.sender != x.creator_wallet {
        return Err(ContractError::Unauthorized{ });
    }

    x.project_status = ProjectStatus::Whitelist;
    x.whitelist = Vec::new();
    PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;
    Ok(Response::new())
}

pub fn try_registerwhitelist(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    project_id: Uint128, 
    card_type: CardType
) 
    -> Result<Response, ContractError> 
{
    let mut x = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    x.whitelist.push( WhitelistState{
        wallet: info.sender,
        card_type: card_type,
        allocation: Uint128::zero(),
        backed: Uint128::zero(),
    });
    PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;
    Ok(Response::new())
}

pub fn try_closewhitelist(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    project_id: Uint128
) 
    -> Result<Response, ContractError> 
{
    let mut x = PROJECTSTATES.load(deps.storage, project_id.u128().into())?;
    if info.sender != x.creator_wallet {
        return Err(ContractError::Unauthorized{ });
    }
    if x.project_status != ProjectStatus::Whitelist {
        return Err(ContractError::NotCorrectStatus{status: x.project_status as u32});
    }

    let backamount;
    if x.backerbacked_amount >= x.project_collected {
        backamount = Uint128::zero()
     } else { 
         backamount = x.project_collected * Uint128::from(UST) - x.backerbacked_amount; 
    }
    
    let mut platium_count = 0;
    let mut gold_count = 0;
    let mut silver_count = 0;
    let mut bronze_count = 0;

    for one in x.whitelist.clone(){
        match one.card_type{
            CardType::Platium => {
                platium_count += 1;
            }
            CardType::Gold => {
                gold_count += 1;
            }
            CardType::Silver => {
                silver_count += 1;
            }
            CardType::Bronze => {
                bronze_count += 1;
            }
            other => {}
        }
    }

    x.holder_ticket = backamount * x.holder_alloc / Uint128::from(100u128) /
        Uint128::from((platium_count*120 + gold_count*50 + silver_count*11 + bronze_count) as u128);

    let community = COMMUNITY.load(deps.storage)?;
    x.community_ticket = backamount * (Uint128::from(100u128) - x.holder_alloc) / Uint128::from(100u128) /
        Uint128::from(community.len() as u128);

    for i in 0..x.whitelist.len(){
        let card_type = x.whitelist[i].card_type.clone();
        match card_type{
            CardType::Platium => {
                x.whitelist[i].allocation = x.holder_ticket * Uint128::from(120u128);
            }
            CardType::Gold => {
                x.whitelist[i].allocation = x.holder_ticket * Uint128::from(50u128);
            }
            CardType::Silver => {
                x.whitelist[i].allocation = x.holder_ticket * Uint128::from(110u128);
            }
            CardType::Bronze => {
                x.whitelist[i].allocation = x.holder_ticket;
            }
            other => {}
        }
    }
    for one in community{
        x.whitelist.push(WhitelistState{
            wallet: one,
            card_type: CardType::Other,
            allocation: x.community_ticket.clone(),
            backed: Uint128::zero()
        })
    }
    x.project_status = ProjectStatus::Fundraising;

    PROJECTSTATES.save(deps.storage, project_id.u128().into(), &x)?;
    Ok(Response::new())
}
