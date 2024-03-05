use fuels::prelude::*;
use orderbook::orderbook_utils::{Orderbook, I64};
use src20_sdk::token_utils::{deploy_token_contract, Asset};

//todo протестировать на ETH и UNI маркетех
//todo было бы удобно если с open_order возвращал order_id
//fixme что значит _n1, p1..? мб лучше назвать _sell, _buy
//todo вынести в отдельный файл имплементации I64
//todo переписать остальные тесты с использованием orderbook_utils как в open_base_token_order_cancel_test
const PRICE_DECIMALS: u64 = 9;

#[tokio::test]
async fn open_base_token_order_cancel_test() {
    //--------------- WALLETS ---------------
    let wallets_config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();
    let admin = &wallets[0];
    let user = &wallets[1];

    let token_contract = deploy_token_contract(&admin).await;
    let btc = Asset::new(admin.clone(), token_contract.contract_id().into(), "BTC");
    let token_contract = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contract.contract_id().into(), "USDC");

    let orderbook = Orderbook::deploy(&admin, usdc.asset_id, usdc.decimals, PRICE_DECIMALS).await;

    // Create Market
    orderbook
        ._create_market(btc.asset_id, btc.decimals as u32)
        .await
        .unwrap();

    let response = orderbook.market_exists(btc.asset_id).await.unwrap();
    assert_eq!(true, response.value);

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();
    assert_eq!(0, response.value.len());

    // SELL 5btc, price 50000
    let price = 50000;
    let btcv: f64 = -5.0;

    let base_price = price * 10u64.pow(PRICE_DECIMALS as u32);
    let base_size_n1 = btc.parse_units(btcv) as i64; //? тут мы имеем i64 а не f64 потому что мы уже домнлжили на scale
    let amount_btc = base_size_n1.abs() as u64;

    // Mint BTC
    btc.mint(admin.address().into(), amount_btc).await.unwrap();
    let balance = admin.get_asset_balance(&btc.asset_id).await.unwrap();
    assert_eq!(balance, amount_btc);

    // Open order
    orderbook
        .open_order(btc.asset_id, base_size_n1, base_price)
        .await
        .unwrap();

    assert_eq!(admin.get_asset_balance(&btc.asset_id).await.unwrap(), 0);

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();

    assert_eq!(1, response.value.len());

    let order_id = response.value.get(0).unwrap();
    let response = orderbook.order_by_id(order_id).await.unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_n1, order.base_size.as_i64());

    // Add btc value to order
    btc.mint(admin.address().into(), amount_btc).await.unwrap();

    orderbook
        .open_order(btc.asset_id, base_size_n1, base_price)
        .await
        .unwrap();

    assert_eq!(admin.get_asset_balance(&btc.asset_id).await.unwrap(), 0);

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();

    assert_eq!(1, response.value.len());

    let order_id = response.value.get(0).unwrap();
    let response = orderbook.order_by_id(order_id).await.unwrap();

    let base_size_n2 = base_size_n1 * 2;

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_n2, order.base_size.as_i64());

    // BUY 5btc, price 5000
    let btcv = 5.0;
    let usdv = 5.0 * price as f64; // 250k usdc

    let base_size_p1 = btc.parse_units(btcv) as i64;
    let quote_size_p1 = usdc.parse_units(usdv) as i64;
    let amount_usdc = quote_size_p1 as u64;

    // Mint USDC
    usdc.mint(admin.address().into(), amount_usdc)
        .await
        .unwrap();

    let balance = admin.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(balance, amount_usdc);

    // Add usdc value to order
    orderbook
        .open_order(btc.asset_id, base_size_p1, base_price)
        .await
        .unwrap();

    let balance = admin.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(balance, amount_usdc);

    let balance = admin.get_asset_balance(&btc.asset_id).await.unwrap();
    assert_eq!(balance, amount_btc);

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();

    assert_eq!(1, response.value.len());

    let order_id = response.value.get(0).unwrap();
    let response = orderbook.order_by_id(order_id).await.unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_n1, order.base_size.as_i64());

    // Mint USDC
    usdc.mint(admin.address().into(), amount_usdc)
        .await
        .unwrap();

    let balance = admin.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(balance, amount_usdc * 2);

    // Add more usdc value to order
    let base_size_p2 = base_size_p1 * 2;

    orderbook
        .open_order(btc.asset_id, base_size_p2.clone(), base_price)
        .await
        .unwrap();

    let balance = admin.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(balance, amount_usdc);

    let balance = admin.get_asset_balance(&btc.asset_id).await.unwrap();
    assert_eq!(balance, amount_btc * 2);

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();
    assert_eq!(1, response.value.len());

    let order_id = response.value.get(0).unwrap();
    let response = orderbook.order_by_id(order_id).await.unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_p1, order.base_size.as_i64());

    // Cancel by not order owner
    orderbook
        .with_account(user)
        .cancel_order(order_id)
        .await
        .expect_err("Order cancelled by another user");

    // Cancel order
    orderbook.cancel_order(order_id).await.unwrap();

    let response = orderbook.orders_by_trader(admin.address()).await.unwrap();
    assert_eq!(0, response.value.len());

    let response = orderbook.order_by_id(order_id).await.unwrap();
    assert!(response.value.is_none());

    let balance = admin.get_asset_balance(&btc.asset_id).await.unwrap();
    assert_eq!(balance, 2 * amount_btc);

    let balance = admin.get_asset_balance(&usdc.asset_id).await.unwrap();
    assert_eq!(balance, 2 * amount_usdc);
}

#[tokio::test]
async fn open_quote_token_order_cancel_by_reverse_order_test() {
    //--------------- WALLETS ---------------
    let wallets_config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();
    let admin = &wallets[0];

    let token_contract = deploy_token_contract(&admin).await;
    let btc = Asset::new(admin.clone(), token_contract.contract_id().into(), "BTC");
    let token_contract = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contract.contract_id().into(), "USDC");

    let orderbook = Orderbook::deploy(&admin, usdc.asset_id, usdc.decimals, PRICE_DECIMALS).await;

    // Create Market
    orderbook
        ._create_market(btc.asset_id, btc.decimals as u32)
        .await
        .unwrap();

    let response = orderbook.market_exists(btc.asset_id).await.unwrap();
    assert_eq!(true, response.value);

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(admin.address())
        .call()
        .await
        .unwrap();

    assert_eq!(0, response.value.len());

    // Mint BTC & USDC

    let usd = 250000;
    let btcv = 5;
    let price = 50000;
    let amount_usdc = usd * 10u64.pow(usdc.decimals.try_into().unwrap());
    let amount_btc = btcv * 10u64.pow(btc.decimals.try_into().unwrap());
    let base_price = price * 10u64.pow(PRICE_DECIMALS as u32);
    let base_size_p1: I64 = I64 {
        value: amount_btc,
        negative: false,
    };
    let base_size_n1: I64 = I64 {
        value: amount_btc,
        negative: true,
    };

    usdc.mint(admin.address().into(), amount_usdc)
        .await
        .unwrap();
    btc.mint(admin.address().into(), amount_btc).await.unwrap();

    assert_eq!(
        admin.get_asset_balance(&usdc.asset_id).await.unwrap(),
        amount_usdc
    );
    assert_eq!(
        admin.get_asset_balance(&btc.asset_id).await.unwrap(),
        amount_btc
    );

    // Open order

    let call_params = CallParameters::default()
        .with_asset_id(usdc.asset_id)
        .with_amount(amount_usdc);

    orderbook
        .instance
        .methods()
        .open_order(btc.asset_id, base_size_p1.clone(), base_price)
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap();

    assert_eq!(admin.get_asset_balance(&usdc.asset_id).await.unwrap(), 0);

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(admin.address())
        .call()
        .await
        .unwrap();

    assert_eq!(1, response.value.len());

    let order_id = response.value.get(0).unwrap();
    let response = orderbook
        .instance
        .methods()
        .order_by_id(*order_id)
        .call()
        .await
        .unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_p1, order.base_size);

    // Cancel order by submitting btc
    let call_params = CallParameters::default()
        .with_asset_id(btc.asset_id)
        .with_amount(amount_btc);

    orderbook
        .instance
        .methods()
        .open_order(btc.asset_id, base_size_n1.clone(), base_price)
        .append_variable_outputs(2)
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap();

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(admin.address())
        .call()
        .await
        .unwrap();

    assert_eq!(0, response.value.len());

    let response = orderbook
        .instance
        .methods()
        .order_by_id(*order_id)
        .call()
        .await
        .unwrap();

    assert!(response.value.is_none());

    assert_eq!(
        admin.get_asset_balance(&usdc.asset_id).await.unwrap(),
        amount_usdc
    );
    assert_eq!(
        admin.get_asset_balance(&btc.asset_id).await.unwrap(),
        amount_btc
    );
}

#[tokio::test]
async fn match_orders_test() {
    //--------------- WALLETS ---------------
    let wallets_config = WalletsConfig::new(Some(5), Some(1), Some(1_000_000_000));
    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();
    let admin = &wallets[0];
    let user1 = &wallets[1];
    let user2 = &wallets[2];

    let token_contract = deploy_token_contract(&admin).await;
    let btc = Asset::new(admin.clone(), token_contract.contract_id().into(), "BTC");
    let token_contract = deploy_token_contract(&admin).await;
    let usdc = Asset::new(admin.clone(), token_contract.contract_id().into(), "USDC");

    let orderbook = Orderbook::deploy(&admin, usdc.asset_id, usdc.decimals, PRICE_DECIMALS).await;

    // Create Market
    orderbook
        ._create_market(btc.asset_id, btc.decimals as u32)
        .await
        .unwrap();

    // Mint BTC & USDC

    let usd = 250000;
    let btcv = 5;
    let price = 50000;
    let amount_usdc = usd * 10u64.pow(usdc.decimals.try_into().unwrap());
    let amount_btc = btcv * 10u64.pow(btc.decimals.try_into().unwrap());
    let base_price = price * 10u64.pow(PRICE_DECIMALS as u32);
    let base_size_p1: I64 = I64 {
        value: amount_btc,
        negative: false,
    };
    let base_size_n1: I64 = I64 {
        value: amount_btc,
        negative: true,
    };

    usdc.mint(user1.address().into(), amount_usdc)
        .await
        .unwrap();
    btc.mint(user2.address().into(), amount_btc).await.unwrap();

    assert_eq!(
        user1.get_asset_balance(&usdc.asset_id).await.unwrap(),
        amount_usdc
    );
    assert_eq!(
        user2.get_asset_balance(&btc.asset_id).await.unwrap(),
        amount_btc
    );

    // Open USDC order

    let call_params = CallParameters::default()
        .with_asset_id(usdc.asset_id)
        .with_amount(amount_usdc);

    orderbook
        .with_account(user1)
        .instance
        .methods()
        .open_order(btc.asset_id, base_size_p1.clone(), base_price)
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap();

    assert_eq!(user1.get_asset_balance(&usdc.asset_id).await.unwrap(), 0);

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(user1.address())
        .call()
        .await
        .unwrap();

    assert_eq!(1, response.value.len());

    let order_id_1 = response.value.get(0).unwrap();
    let response = orderbook
        .instance
        .methods()
        .order_by_id(*order_id_1)
        .call()
        .await
        .unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_p1, order.base_size);

    // Open BTC order

    let call_params = CallParameters::default()
        .with_asset_id(btc.asset_id)
        .with_amount(amount_btc);

    orderbook
        .with_account(user2)
        .instance
        .methods()
        .open_order(btc.asset_id, base_size_n1.clone(), base_price)
        .call_params(call_params)
        .unwrap()
        .call()
        .await
        .unwrap();

    assert_eq!(user2.get_asset_balance(&btc.asset_id).await.unwrap(), 0);

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(user2.address())
        .call()
        .await
        .unwrap();

    assert_eq!(1, response.value.len());

    let order_id_2 = response.value.get(0).unwrap();
    let response = orderbook
        .instance
        .methods()
        .order_by_id(*order_id_2)
        .call()
        .await
        .unwrap();

    let order = response.value.unwrap();
    assert_eq!(base_price, order.base_price);
    assert_eq!(base_size_n1, order.base_size);

    // Match orders
    orderbook
        .instance
        .methods()
        .match_orders(*order_id_2, *order_id_1)
        .append_variable_outputs(2)
        .call()
        .await
        .unwrap();

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(user1.address())
        .call()
        .await
        .unwrap();

    assert_eq!(0, response.value.len());

    let response = orderbook
        .instance
        .methods()
        .orders_by_trader(user2.address())
        .call()
        .await
        .unwrap();

    assert_eq!(0, response.value.len());

    assert_eq!(
        user2.get_asset_balance(&usdc.asset_id).await.unwrap(),
        amount_usdc
    );
    assert_eq!(
        user1.get_asset_balance(&btc.asset_id).await.unwrap(),
        amount_btc
    );
}
