use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::currency::currency_client::CurrencyClient;

tonic::include_proto!("currency_proto");

lazy_static::lazy_static! {
    static ref CURRENCY_CLIENT: Arc<RwLock<Option<CurrencyClient<Channel>>>> = {
        Arc::new(RwLock::new(None))
    };
}

pub async fn init_currency_client() {
    let server_url: String = match std::env::var("SERVER_URL") {
        Ok(val) => val,
        Err(_e) => "http://127.0.0.1:3088".to_string(),
    };

    let client_ref = Arc::clone(&CURRENCY_CLIENT);
    loop {
        let cli = CurrencyClient::connect(server_url.clone()).await;
        match cli {
            Ok(client) => {
                tracing::info!("✅Connection to the gRPC service currency is successful!");
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

pub async fn get_currency_grpc() -> CurrencyDto {
    let request = tonic::Request::new(CurrencyRequest {
        input: "".to_string(),
    });
    let cur = get_currency_client().await.get_currency(request).await;
    match cur {
        Ok(t) => {
            let res = t.into_inner();
            tracing::debug!("Currency: '{:?}' from service", res);
            CurrencyDto::build_from_msg(res)
        }
        Err(e) => {
            tracing::error!("Error getting currency: {:?}", e);
            CurrencyDto::default()
        }
    }
}

impl CurrencyMsg {
    pub fn build_from_currency(cur: Option<CurrencyDto>) -> Self {
        match cur {
            Some(c) => CurrencyMsg {
                created_at: c.created_at.unwrap().to_string(),
                dkk: c.dkk,
                eur: c.eur,
                gbp: c.gbp,
                btc: c.btc,
                eth: c.eth,
                ts: (c.ts / 1000) as i32,
            },
            None => CurrencyMsg::default(),
        }
    }
}

async fn get_currency_client() -> CurrencyClient<Channel> {
    let client_ref = Arc::clone(&CURRENCY_CLIENT);
    let client = client_ref.read().await.clone();
    client.expect("Currency service not found")
}

#[derive(Debug, FromRow, Default, Clone)]
pub struct CurrencyDto {
    pub ts: i64,
    pub dkk: f32,
    pub eur: f32,
    pub gbp: f32,
    pub btc: f32,
    pub eth: f32,
    pub created_at: Option<DateTime<Utc>>,
}

impl CurrencyDto {
    pub fn build_from_msg(msg: CurrencyMsg) -> Self {
        let t = msg.ts as i64;
        CurrencyDto {
            ts: t,
            dkk: msg.dkk,
            eur: msg.eur,
            gbp: msg.gbp,
            btc: msg.btc,
            eth: msg.eth,
            created_at: None,
        }
    }
}
