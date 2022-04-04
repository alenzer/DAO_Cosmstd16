use cosmwasm_std::{Uint128, Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::staking::{CardType};

//------------Config---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub wefund: Addr,
    pub anchor_market: Addr,
    pub aust_token: Addr,
    pub vesting_contract: Addr,
}

//-------------backer states---------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BackerState{
    pub backer_wallet: Addr,
    pub ust_amount: Coin,
    pub aust_amount: Coin,
    pub otherchain: String,
    pub otherchain_wallet: String,
}
//--------------Vote---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote{
    pub wallet: Addr,
    pub voted: bool,
}

//--------------Milestone---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Milestone{
    pub milestone_step: Uint128,
    pub milestone_name: String,
    pub milestone_type: String,
    pub milestone_description: String,
    pub milestone_startdate: String,
    pub milestone_enddate: String,
    pub milestone_amount: Uint128,
    pub milestone_status: Uint128, //0:voting, 1:releasing 2:released
    pub milestone_votes: Vec<Vote>,
}
//------------Team Description-------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TeamMember{
    pub teammember_description: String,
    pub teammember_linkedin: String,
    pub teammember_role: String,
    pub teammember_name: String,
}
//--------------Milestone---------------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingParameter{
    pub stage_title: String,
    pub stage_price: Uint128,
    pub stage_amount: Uint128,
    pub stage_soon: Uint128,
    pub stage_after: Uint128,
    pub stage_period: Uint128   
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistState{
    pub wallet: Addr,
    pub card_type: CardType,
    pub allocation: Uint128,
    pub backed: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProjectStatus{
    WefundVote,
    Whitelist,
    Fundraising,
    Releasing,
    Done,
    Fail
}
//------------ project state--------------------------------
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProjectState{
//---------mata data----------------------------------------------------------
    pub project_company: String,
    pub project_title: String,
    pub project_description: String,
    pub project_ecosystem: String,
    pub project_createddate: String,
    pub project_saft: String,
    pub project_logo: String,
    pub project_whitepaper: String,
    pub project_website: String,
    pub project_email: String,

    pub country: String,
    pub cofounder_name: String,
    pub service_wefund: String,
    pub service_charity: String,
    pub professional_link: String,
//------------------------------------------------------------------------------
    pub project_id: Uint128,
    pub creator_wallet: Addr,
    pub project_collected: Uint128,

    //0:wefund voting 1:fundrasing 2:releasing 3:done 4:fail
    pub project_status: ProjectStatus, 
    pub fundraising_stage: Uint128, 

    pub backerbacked_amount: Uint128,
    pub aust_amount: Uint128,
//---------backer states for 50% of collected------------------------    
    pub backer_states: Vec<BackerState>,

//----------milestone states-----------------------------------------
    pub milestone_states: Vec<Milestone>,
    pub project_milestonestep: Uint128, 
//---------team members-----------------------------------------------
    pub teammember_states: Vec<TeamMember>,
//---------vesting-----------------------------------------------
    pub vesting: Vec<VestingParameter>,

    pub token_addr: Addr,
//---------whitelist-----------------------------
    pub whitelist: Vec<WhitelistState>,
    pub holder_alloc: Uint128,
    pub holder_ticket: Uint128,
    pub community_ticket: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub wefund: Option<String>,
    pub anchor_market: Option<String>,
    pub aust_token: Option<String>,
    pub vesting_contract: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetConfig { admin:Option<String>,  wefund: Option<String>, 
        anchor_market: Option<String>, aust_token:Option<String> , 
        vesting_contract:Option<String>},
    AddProject { 
        project_id: Uint128,
        project_company: String,
        project_title: String,
        project_description: String,
        project_ecosystem: String,
        project_createddate: String,
        project_saft: String,
        project_logo: String,
        project_whitepaper: String,
        project_website: String,
        project_email: String,
        creator_wallet: String,
        project_collected: Uint128,
        project_milestones: Vec<Milestone>,
        project_teammembers: Vec<TeamMember>,
        vesting: Vec<VestingParameter>,
        token_addr: String,

        country: String,
        cofounder_name: String,
        service_wefund: String,
        service_charity: String,
        professional_link: String
    },
    RemoveProject{project_id: Uint128 },

    Back2Project { project_id: Uint128, backer_wallet: String, 
        fundraising_stage: Uint128, token_amount: Uint128, 
        otherchain:String, otherchain_wallet:String},

    CompleteProject{ project_id: Uint128 },
    FailProject{project_id: Uint128 },

    TransferAllCoins{wallet: String},

    AddCommunitymember{wallet: String},
    RemoveCommunitymember{wallet: String},

    WefundApprove{project_id:Uint128},
    SetFundraisingStage{project_id: Uint128, stage: Uint128},
    
    SetMilestoneVote{project_id: Uint128, wallet:String, voted: bool},

    ReleaseMilestone{project_id: Uint128},

    SetProjectStatus{project_id: Uint128, status: Uint128},

    OpenWhitelist{project_id: Uint128, holder_alloc: Uint128},
    RegisterWhitelist{project_id: Uint128, card_type: CardType},
    CloseWhitelist{project_id: Uint128}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig{},
    GetAllProject{},
    GetProject { project_id:Uint128 },
    GetBacker{ project_id:Uint128},
    GetBalance{ wallet:String },
    GetCommunitymembers{},
}

