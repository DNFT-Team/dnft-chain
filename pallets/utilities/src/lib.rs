#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#![allow(clippy::string_lit_as_bytes)]

use frame_support::dispatch;
use sp_std::{
    cmp::{Eq, PartialEq},
    ops::Not,
    prelude::*,
};

use codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::{DispatchResult, RuntimeDebug};

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Did {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct NFTId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ClassId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct CollectionId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct TokenId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AIDataId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AIModelId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AuctionId {
    pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProposalId {
    pub did: [u8; 32],
}

/// NFT Class
#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct ClassInfo<AccountId> {
    pub name: Vec<u8>,
    pub info: Vec<u8>,
    pub total_supply: u64,
    pub issuer: AccountId,
}

/// NFT
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum NFTStatus {
    Normal = 0,
    Burned,
    Offered,
    InCollection,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct NFTInfo<AccountId, Balance> {
    pub class_id: ClassId,
    pub index: u64,
    pub info: Vec<u8>,
    pub metadata: Vec<u8>,
    pub owner: AccountId,
    pub issuer: AccountId,
    pub price: Balance,
    pub status: NFTStatus,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct NFTSource<ClassId> {
    pub class_id: ClassId,
    pub amount: u64,
    pub nfts_indexs: Vec<u64>,
}

pub trait NFT721Manager<AccountId, Balance> {
    // Class
    fn issue_nft_class(
        name: Vec<u8>,
        info: Vec<u8>,
        total_supply: u64,
        issuer: AccountId,
    ) -> DispatchResult;

    fn get_class(class_id: ClassId) -> Option<ClassInfo<AccountId>>;

    // NFT
    fn mint_nft(
        class_id: ClassId,
        info: Vec<u8>,
        metadata: Vec<u8>,
        price: Balance,
        miner: AccountId,
    ) -> Option<NFTId>;

    fn get_nft(nft_id: NFTId) -> Option<NFTInfo<AccountId, Balance>>;

    // Todo safeTransfer
    fn transfer_single_nft(from: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

    fn destroy_single_nft(who: AccountId, nft_id: NFTId) -> DispatchResult;
}

pub trait NFT1155Manager<AccountId, Balance> {
    // nft class
    fn issue_nft_class(
        name: Vec<u8>,
        info: Vec<u8>,
        supply: u64,
        issuer: AccountId,
    ) -> DispatchResult;

    fn get_class(class_id: ClassId) -> Option<ClassInfo<AccountId>>;

    // NFT
    fn mint_nft(
        class_id: ClassId,
        info: Vec<u8>,
        metadata: Vec<u8>,
        price: Balance,
        miner: AccountId,
    ) -> Option<NFTId>;

    fn get_nft(nft_id: NFTId) -> Option<NFTInfo<AccountId, Balance>>;

    fn get_nft_by_index(class_id: ClassId, index: u64) -> Option<NFTId>;

    // fn owned_nfts(account: AccountId) -> Vec<NFTSource<ClassId>>;

    // Todo safeTransfer
    fn transfer_single_nft(from: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

    fn transfer_batch_nft(
        from: AccountId,
        to: AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult;

    fn approve_single_nft(who: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

    fn approve_batch_nft(
        who: AccountId,
        to: AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult;

    fn destroy_single_nft(who: AccountId, nft_id: NFTId) -> DispatchResult;

    fn destroy_batch_nft(who: AccountId, class_id: ClassId, amount: u64) -> DispatchResult;
}

/// Collection
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum CollectionStatus {
    Normal = 0,
    Decoupled,
    Burned,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct Collection<AccountId, Balance, CollectionStatus, ClassId> {
    pub name: Vec<u8>,
    pub symbol: Vec<u8>,
    pub info: Vec<u8>,
    pub owner: AccountId,
    pub issuer: AccountId,
    pub price: Balance,
    pub source: Vec<NFTSource<ClassId>>,
    pub status: CollectionStatus,
}

pub trait NFT2006Manager<AccountId, Balance> {
    // nft class
    fn issue_nft_class(
        name: Vec<u8>,
        info: Vec<u8>,
        supply: u64,
        issuer: AccountId,
    ) -> DispatchResult;

    fn get_class(class_id: ClassId) -> Option<ClassInfo<AccountId>>;

    // NFT
    fn mint_nft(
        class_id: ClassId,
        info: Vec<u8>,
        metadata: Vec<u8>,
        price: Balance,
        miner: AccountId,
    ) -> Option<NFTId>;

    fn get_nft(nft_id: NFTId) -> Option<NFTInfo<AccountId, Balance>>;

    fn get_nft_by_index(class_id: ClassId, index: u64) -> Option<NFTId>;

    // fn owned_nfts(account: AccountId) -> Vec<NFTSource<ClassId>>;

    // Todo safeTransfer
    fn transfer_single_nft(from: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

    fn transfer_batch_nft(
        from: AccountId,
        to: AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult;

    fn approve_single_nft(who: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

    fn approve_batch_nft(
        who: AccountId,
        to: AccountId,
        class_id: ClassId,
        amount: u64,
    ) -> DispatchResult;

    fn destroy_single_nft(who: AccountId, nft_id: NFTId) -> DispatchResult;

    fn destroy_batch_nft(who: AccountId, class_id: ClassId, amount: u64) -> DispatchResult;

    // Collection
    fn coupled_collection(
        name: Vec<u8>,
        symbol: Vec<u8>,
        owner: AccountId,
        info: Vec<u8>,
        price: Balance,
        source: Vec<NFTSource<ClassId>>,
    ) -> DispatchResult;

    fn get_collection(
        collection_id: CollectionId,
    ) -> Option<Collection<AccountId, Balance, CollectionStatus, ClassId>>;

    fn owned_collections(account: AccountId) -> Vec<CollectionId>;

    fn decoupled_collection(who: AccountId, collection_id: CollectionId) -> DispatchResult;

    fn transfer_collection(
        who: AccountId,
        from: AccountId,
        to: AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult;

    fn destroy_collection(who: AccountId, collection_id: CollectionId) -> DispatchResult;

    fn approve_collection(
        who: AccountId,
        to: AccountId,
        collection_id: CollectionId,
    ) -> DispatchResult;

    fn set_approval_for_all(who: AccountId, to: AccountId, approved: bool) -> DispatchResult;
}
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum NFTType {
    NFT721 = 0,
    NFT1155,
    NFT2006,
}
/// DAO
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ProposalTheme {
    ChangeDAOTax = 0,
    DAOAcc,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ProposalStatus {
    Created = 0,
    Passed,
    Failed,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Proposal<AccountId, Balance> {
    pub owner: AccountId,
    pub theme: ProposalTheme,
    pub value_number: Option<u64>,
    pub value_string: Option<Vec<u8>>,
    pub value_money: Option<Balance>,
    pub min_to_succeed: u64,
    pub vote_yes: u64,
    pub vote_no: u64,
    pub deadline: u64,
    pub status: ProposalStatus,
}

pub trait DAOManager<AccountId, Balance> {
    fn get_dao_account() -> AccountId;
    fn get_dao_tax() -> Balance;
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct AIModelHighlight {
    pub theme: Vec<u8>,
    pub info: Vec<u8>,
    pub score: u64,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ModelLanguage {
    Python = 0,
    Js,
    Java,
    Matlab,
    Lisp,
    Prolog,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct AIModel<AccountId, Moment> {
    pub creator: AccountId,
    pub title: Vec<u8>,
    pub language: ModelLanguage,
    pub framwork: Vec<u8>,
    pub stars: u64,
    pub timestamp: Moment,
    pub highlight: Vec<AIModelHighlight>,
    pub nft_id: Option<NFTId>,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DataIndustry {
    Business = 0,
    Internet,
    Finance,
    Healthcare,
    PoliticsAndCulture,
    ComputerScience,
    EngineeringAndTransportation,
    Safety,
    FashionAndArt,
    NatureScience,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DataTechnology {
    ComputerVision = 0,
    NaturalLanguageProcessing,
    Classification,
    ObjectDetection,
    SpeechRecognition,
    MachineLearning,
    Modeling,
    RecommendationSystems,
    DeepLearning,
    VideoData,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum DataResource {
    PublicData = 0,
    MyData,
    Burned,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct AIData<AccountId, Moment> {
    pub creator: AccountId,
    pub industry: DataIndustry,
    pub technology: DataTechnology,
    pub resource: DataResource,
    pub stars: u64,
    pub timestamp: Moment,
    pub nft_id: Option<NFTId>,
    pub collection_id: Option<CollectionId>,
}

pub type BufferIndex = u8;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ValueStruct {
    pub integer: u32,
    pub boolean: bool,
}

pub trait CommonManager<AccountId> {
    /// did
    fn generate_did(from: AccountId, nonce: u64) -> Did;
    fn generate_hash(from: AccountId, nonce: u64) -> H256;
    /// ringbuffer
    fn add_to_queue(id: u32, integer: u32, boolean: bool);
    fn add_multiple(id: u32, integers: Vec<u32>, boolean: bool);
    fn pop_from_queue(id: u32);
    fn get_buffer_range(id: u32) -> (BufferIndex, BufferIndex);
    fn get_buffer_value(id: u32, index: BufferIndex) -> ValueStruct;
}

/// token
#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Token<AccountId> {
    pub tid: Did,
    pub owner: AccountId,
    pub symbol: Vec<u8>,
    pub total_supply: u64,
}

pub trait TokenManager<AccountId> {
    // issue
    fn issue(from: AccountId, total_supply: u64, symbol: Vec<u8>) -> Did;

    // transfer
    fn transfer(
        from: AccountId,
        to: AccountId,
        token_id: Did,
        value: u64,
        memo: Option<Vec<u8>>,
    ) -> DispatchResult;

    // transfer
    fn static_transfer_in(from: AccountId, to: Did, token_id: Did, value: u64) -> DispatchResult;
    fn static_transfer_out(from: Did, to: AccountId, token_id: Did, value: u64) -> DispatchResult;

    // freeze
    fn freeze(from: AccountId, token_id: Did, value: u64) -> DispatchResult;

    // unfreeze
    fn unfreeze(from: AccountId, token_id: Did, value: u64) -> DispatchResult;

    // query
    fn balance_of(from: AccountId, token_id: Did) -> u64;
    fn static_balance_of(from: Did, token_id: Did) -> u64;
    fn owner_of(token_id: Did) -> Option<AccountId>;
    fn ensure_free_balance(sender: AccountId, token_id: Did, amount: u64) -> DispatchResult;
}
/// order
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum OrderType {
    Buy = 0,
    Sell,
}

impl Not for OrderType {
    type Output = OrderType;

    fn not(self) -> Self::Output {
        match self {
            OrderType::Sell => OrderType::Buy,
            OrderType::Buy => OrderType::Sell,
        }
    }
}
/// trade
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum TradeMethod {
    Auction = 0,
    AMMOrder,
    OrderBook,
    P2POrder,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct TradePair {
    pub base: Did,
    pub quote: Did,
    pub method: TradeMethod,
    pub matched_price: u64,
    pub one_day_trade_volume: u64,
    pub one_day_highest_price: u64,
    pub one_day_lowest_price: u64,
}
pub trait TradePairManager<AccountId> {
    // create_trade_pair
    fn create_trade_pair(
        sender: AccountId,
        base: Did,
        quote: Did,
        method: TradeMethod,
        matched_price: Option<u64>,
    ) -> Result<Did, dispatch::DispatchError>;

    // update_trade_pair
    fn transfer(sender: AccountId, tpid: Did, new_trade_pair: TradePair) -> DispatchResult;

    // get_trade_pair
    fn get_trade_pair(tpid: Did) -> Option<TradePair>;
    //get_trade_pair_id_by_base_quote
    fn get_trade_pair_id_by_base_quote(base: Did, quote: Did) -> Option<Did>;
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub enum OrderStatus {
    Created,
    PartialFilled,
    Filled,
    Canceled,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct LiquidityPool {
    pub tpid: Did,
    pub token0: Did,
    pub token1: Did,
    pub token0_amount: u64,
    pub token1_amount: u64,
    pub k_last: u64, // k_last = token0_amount * token1_amount
    pub swap_price_last: u64,
    pub swap_price_highest: u64,
    pub swap_price_lowest: u64,
    pub token0_trade_volume_total: u64,
    pub token1_trade_volume_total: u64,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct AmmOrder<AccountId> {
    pub lpid: Did,
    pub owner: AccountId,
    pub token_have: Did,
    pub token_have_amount: u64,
    pub token_want: Did,
    pub token_want_amount: u64,
    pub token_swap_price: u64,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct P2POrderMaker<AccountId, Moment> {
    pub tpid: Did,
    pub maker: AccountId,
    pub order_type: OrderType,
    pub volume: u64,
    pub price: u64,
    pub status: OrderStatus,
    pub time: Moment,
    pub left_volume: u64,
    pub locked_volume: u64,
    pub taked_volume: u64,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct OrderQueueInfo {
    pub oindex: u32,
    pub price: u32,
    pub time: u32,
    pub volume: u32,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct LimitOrder<AccountId, Moment> {
    pub tpid: Did,
    pub owner: AccountId,
    pub price: u64,
    pub amount: u64,
    pub created_time: Moment,
    pub remained_amount: u64,
    pub otype: OrderType,
    pub status: OrderStatus,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Trade<AccountId> {
    pub tpid: Did,
    pub buyer: AccountId,
    pub seller: AccountId,
    pub price: u64,
    pub otype: OrderType,
    pub base_amount: u64,
    pub quote_amount: u64,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum AuctionType {
    EighshAuction = 0,
    DutchAuction,
    SealedAuction,
    DoubleAuction,
    HabergerTaxAuction,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum AuctionStatus {
    Created = 0,
    Canceled,
    Confirmed,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Auction<AccountId, Balance, Moment> {
    pub owner: AccountId,
    pub auction_type: AuctionType,
    pub nft_type: NFTType,
    pub nft_id: NFTId,
    pub base_price: Option<Balance>,
    pub start_time: Option<Moment>,
    pub end_time: Option<Moment>,
    pub status: AuctionStatus,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct BidInfo<AccountId, Balance, Moment> {
    pub bidder: AccountId,
    pub price: Balance,
    pub time: Moment,
    pub is_legal: bool,
    pub is_winner: bool,
}
