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
pub struct NFTSId {
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
pub struct AuctionId {
	pub did: [u8; 32],
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct ProposalId {
	pub did: [u8; 32],
}

/// NFT Class
#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct ClassInfo<AccountId, BlockNumber> {
	pub name: Vec<u8>,
	pub metadata: Vec<u8>,
	pub info: Vec<u8>,
	pub supply: u64,
    pub issuer: AccountId,
}

/// NFT
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum NFTStatus {
	Normal = 0,
	Destroyed,
	InCollection,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct NFT<AccountId, Balance, NFTSId, NFTStatus> {
	pub nfts_id: NFTSId,
	pub nfts_index: u64,
	pub info: Vec<u8>,
	pub owner: AccountId,
    pub price: Balance,
	pub status: NFTStatus,
	pub approvers: Vec<AccountId>,
}

/// Collection
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum CollectionStatus {
	Normal = 0,
	Destroyed,
	Decoupled,
}

#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct LifeStage<BlockNumber> {
	pub name: Vec<u8>,
	pub period: BlockNumber,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct NFTS<AccountId, BlockNumber> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub supply: u64,
	pub stage: Vec<LifeStage<BlockNumber>>,
	pub issuer: AccountId,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct NFTSource<NFTSId> {
	pub nfts_id: NFTSId,
	pub amount: u64,
	pub nfts_indexs: Vec<u64>,
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone)]
pub struct Collection<AccountId, CollectionStatus, NFTSId> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub info: Vec<u8>,
	pub owner: AccountId,
	pub source: Vec<NFTSource<NFTSId>>,
	pub status: CollectionStatus,
	pub approvers: Vec<AccountId>,
}


pub trait NFTManager<AccountId, BlockNumber> {
	// NFTS
	fn issue_nfts(
		name: Vec<u8>,
		symbol: Vec<u8>,
		info: Vec<u8>,
		supply: u64,
		stage: Vec<LifeStage<BlockNumber>>,
		issuer: AccountId,
	) -> DispatchResult;

	fn get_nfts(nfts_id: NFTSId) -> Option<NFTS<AccountId, BlockNumber>>;

	// NFT
	fn mint_nft(nfts_id: NFTSId, miner: AccountId, info: Vec<u8>) -> Option<NFTId>;

	fn get_nft(nft_id: NFTId) -> Option<NFT<AccountId, NFTSId, NFTStatus>>;

	fn get_nfts_member_index(nfts_id: NFTSId) -> u64;

	fn get_nft_by_index(nfts_id: NFTSId, nfts_index: u64) -> Option<NFTId>;

	fn owned_nfts(account: AccountId) -> Vec<NFTSource<NFTSId>>;

	// Todo safeTransfer
	fn transfer_single_nft(
		who: AccountId,
		from: AccountId,
		to: AccountId,
		nft_id: NFTId,
	) -> DispatchResult;

	fn transfer_batch_nft(
		who: AccountId,
		from: AccountId,
		to: AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult;

	fn approve_single_nft(who: AccountId, to: AccountId, nft_id: NFTId) -> DispatchResult;

	fn approve_batch_nft(
		who: AccountId,
		to: AccountId,
		nfts_id: NFTSId,
		amount: u64,
	) -> DispatchResult;

	fn destroy_single_nft(who: AccountId, nft_id: NFTId) -> DispatchResult;

	fn destroy_batch_nft(who: AccountId, nfts_id: NFTSId, amount: u64) -> DispatchResult;

	// Collection
	fn coupled_collection(
		name: Vec<u8>,
		symbol: Vec<u8>,
		owner: AccountId,
		info: Vec<u8>,
		source: Vec<NFTSource<NFTSId>>,
	) -> DispatchResult;

	fn get_collection(
		collection_id: CollectionId,
	) -> Option<Collection<AccountId, CollectionStatus, NFTSId>>;

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
    fn issue(from: AccountId, total_supply: u64, symbol: Vec<u8>) -> DispatchResult;

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

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Proposal<AccountId> {
    pub owner: AccountId,
    pub name: Vec<u8>,
    pub content: Vec<u8>,
    pub min_to_succeed: u64,
    pub vote_yes: u64,
    pub vote_no: u64,
    pub deadline: u64,
}

#[derive(Encode, Decode, PartialEq, Eq, Clone, RuntimeDebug)]
pub struct Auction<AccountId> {
    pub owner: AccountId,
    pub name: Vec<u8>,
    pub start: u64,
    pub end: u64,
}




