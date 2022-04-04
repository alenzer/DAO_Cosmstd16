use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Coin, StdResult, Storage};
use cw_storage_plus::{Item, Map, U128Key};

use Interface::staking::{CardType};
use Interface::wefund::{ExecuteMsg, InstantiateMsg, Config, ProjectState, BackerState,
    VestingParameter,TeamMember, ProjectStatus, Milestone, Vote, WhitelistState
};

pub const CONFIG: Item<Config> = Item::new("config");

pub const PROJECT_SEQ: Item<Uint128> = Item::new("prj_seq");
pub const PROJECTSTATES: Map<U128Key, ProjectState> = Map::new("prj");

pub fn save_projectstate(store: &mut dyn Storage, _prj: &mut ProjectState) 
    -> StdResult<()> 
{
    // increment id if exists, or return 1
    let id = PROJECT_SEQ.load(store)?;
    let id = id.checked_add(Uint128::new(1))?;
    PROJECT_SEQ.save(store, &id)?;

    _prj.project_id = id.clone();
    PROJECTSTATES.save(store, id.u128().into(), &_prj)
}

//------------community array------------------------------------------------
pub const COMMUNITY: Item<Vec<Addr>> = Item::new("community");

//------------Profit------------------------------------------------------------
pub const PROFIT: Item<Uint128> = Item::new("profit");

//------------FOR REPLY-----------------------------------------
pub const PROJECT_ID: Item<Uint128> = Item::new("project id");
pub const AUST_AMOUNT: Item<Uint128> = Item::new("aust amount");
pub const UUSD_AMOUNT: Item<Uint128> = Item::new("ust amount");
