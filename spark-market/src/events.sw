library;

use ::data_structures::{
    asset_type::AssetType,
    limit_type::LimitType,
    order_type::OrderType,
    protocol_fee::ProtocolFee,
};

pub struct DepositEvent {
    pub amount: u64,
    pub asset: AssetId,
    pub user: Identity,
    pub liquid_base: u64,
    pub liquid_quote: u64,
}

pub struct OpenOrderEvent {
    pub amount: u64,
    pub asset: AssetId,
    pub order_type: OrderType,
    pub order_id: b256,
    pub price: u64,
    pub user: Identity,
    pub liquid_base: u64,
    pub liquid_quote: u64,
}

pub struct CancelOrderEvent {
    pub order_id: b256,
    pub user: Identity,
    pub liquid_base: u64,
    pub liquid_quote: u64,
}

pub struct TradeOrderEvent {
    pub base_sell_order_id: b256,
    pub base_buy_order_id: b256,
    pub base_sell_order_limit: LimitType,
    pub base_buy_order_limit: LimitType,
    pub order_matcher: Identity,
    pub trade_size: u64,
    pub trade_price: u64,
    pub block_height: u32,
    pub tx_id: b256,
    pub order_seller: Identity,
    pub order_buyer: Identity,
    pub s_account_liquid_base: u64,
    pub s_account_liquid_quote: u64,
    pub b_account_liquid_base: u64,
    pub b_account_liquid_quote: u64,
}

pub struct WithdrawEvent {
    pub amount: u64,
    pub asset: AssetId,
    pub user: Identity,
    pub liquid_base: u64,
    pub liquid_quote: u64,
}

pub struct SetEpochEvent {
    pub epoch: u64,
    pub epoch_duration: u64,
}

pub struct SetProtocolFeeEvent {
    pub protocol_fee: Vec<ProtocolFee>,
}

pub struct SetMatcherRewardEvent {
    pub amount: u64,
}
