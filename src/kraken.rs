use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::kraken::kraken_client::KrakenClient;

tonic::include_proto!("kraken_proto");

lazy_static::lazy_static! {
    static ref KRAKEN_CLIENT: Arc<RwLock<Option<KrakenClient<Channel>>>> = {
        Arc::new(RwLock::new(None))
    };
}

pub async fn init_kraken_client() {
    let server_url: String = match std::env::var("SERVER_URL") {
        Ok(val) => val,
        Err(_) => "http://127.0.0.1:3088".to_string(),
    };
    let client_ref = Arc::clone(&KRAKEN_CLIENT);
    loop {
        let cli = KrakenClient::connect(server_url.clone()).await;
        match cli {
            Ok(client) => {
                tracing::info!("✅Connection to the gRPC service kraken is successful!");
                let mut guard = client_ref.write().await;
                guard.replace(client);
                // Add your logic to handle successful connection
                break;
            }
            Err(e) => {
                tracing::info!(
                    "Failed to connect to gRPC service: {}. Retrying in 2 seconds...",
                    e
                );
                std::thread::sleep(Duration::new(2, 0));
            }
        }
    }
}

pub async fn get_monday_grpc() -> KrakenTickerDto {
    let request = tonic::Request::new(KrakenRequest { interval: 7 });
    let tic = get_kraken_client().await.ticker(request).await;
    match tic {
        Ok(t) => {
            let res = t.into_inner();
            tracing::debug!("Monday: '{:?}' from service", res.last_price);
            KrakenTickerDto::build_from_msg(res)
        }
        Err(e) => {
            tracing::error!("Error getting monday price: {:?}", e);
            KrakenTickerDto::default()
        }
    }
}

pub async fn get_ticker_day_first() -> KrakenTickerDto {
    let request = tonic::Request::new(KrakenRequest { interval: 1 });
    let response = get_kraken_client().await.ticker_day(request).await;
    let mut stream = response.unwrap().into_inner();
    if let Some(res) = stream.message().await.unwrap() {
        let res = KrakenTickerDto::build_from_msg(res);
        tracing::debug!("First day = {:?}", res);
        return res;
    }
    KrakenTickerDto::default()
}

async fn get_kraken_client() -> KrakenClient<Channel> {
    let client_ref = Arc::clone(&KRAKEN_CLIENT);
    let client = client_ref.read().await.clone();
    client.expect("Kraken service not found")
}

impl KrakenTickerMsg {
    pub fn build_from_dto(dto: KrakenTickerDto) -> Self {
        KrakenTickerMsg {
            created_at: dto.created_at.unwrap_or_default().to_string(),
            last_price: dto.last_price,
            last_volume: dto.last_volume,
            volume_today: dto.volume_today,
            volume_24_hours: dto.volume_24_hours,
            trades_today: dto.trades_today,
            trades_24_hours: dto.trades_24_hours,
            ask_price: dto.ask_price,
            ask_whole_lot_volume: dto.ask_whole_lot_volume,
            ask_lot_volume: dto.ask_lot_volume,
            bid_price: dto.bid_price,
            bid_whole_lot_volume: dto.bid_whole_lot_volume,
            bid_lot_volume: dto.bid_lot_volume,
            id: dto.id,
        }
    }
}

impl KrakenOhlcMsg {
    pub fn new() -> Self {
        KrakenOhlcMsg {
            ts: Utc::now().to_string(),
            unix_time: 42,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            vwap: 0.0,
            volume: 0.0,
            count: 0,
        }
    }
    pub fn build_from_ohlc(ohlc: &OhlcRow) -> Self {
        KrakenOhlcMsg {
            ts: crate::unix_to_str(ohlc.unix_time),
            unix_time: ohlc.unix_time,
            open: ohlc.open.parse::<f32>().unwrap_or_default(),
            high: ohlc.high.parse::<f32>().unwrap_or_default(),
            low: ohlc.low.parse::<f32>().unwrap_or_default(),
            close: ohlc.close.parse::<f32>().unwrap_or_default(),
            vwap: ohlc.vwap.parse::<f32>().unwrap_or_default(),
            volume: ohlc.volume.parse::<f32>().unwrap_or_default(),
            count: ohlc.count as i32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcRow {
    pub unix_time: i32,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub vwap: String,
    pub volume: String,
    pub count: u32,
}

#[derive(Debug, FromRow, Default, Clone)]
pub struct KrakenTickerDto {
    pub id: i32,
    pub last_price: f32,
    pub last_volume: f32,
    pub volume_today: f32,
    pub volume_24_hours: f32,
    pub trades_today: i32,
    pub trades_24_hours: i32,
    pub ask_price: f32,
    pub ask_whole_lot_volume: i32,
    pub ask_lot_volume: f32,
    pub bid_price: f32,
    pub bid_whole_lot_volume: i32,
    pub bid_lot_volume: f32,
    pub created_at: Option<DateTime<Utc>>,
}

impl KrakenTickerDto {
    pub fn build_from_msg(msg: KrakenTickerMsg) -> Self {
        let dt = msg.created_at.parse::<DateTime<Utc>>();
        KrakenTickerDto {
            id: msg.id,
            last_price: msg.last_price,
            last_volume: msg.last_volume,
            volume_today: msg.volume_today,
            volume_24_hours: msg.volume_24_hours,
            trades_today: msg.trades_today,
            trades_24_hours: msg.trades_24_hours,
            ask_price: msg.ask_price,
            ask_whole_lot_volume: msg.ask_whole_lot_volume,
            ask_lot_volume: msg.ask_lot_volume,
            bid_price: msg.bid_price,
            bid_whole_lot_volume: msg.bid_whole_lot_volume,
            bid_lot_volume: msg.bid_lot_volume,
            created_at: Some(dt.unwrap_or_default()),
        }
    }
}
