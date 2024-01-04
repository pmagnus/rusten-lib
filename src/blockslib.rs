use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::blockslib::blocks_client::BlocksClient;
tonic::include_proto!("blocks_proto");

lazy_static::lazy_static! {
    static ref BLOCKS_CLIENT: Arc<RwLock<Option<BlocksClient<Channel>>>> = {
        Arc::new(RwLock::new(None))
    };
}

pub async fn get_block(height: i64) -> BlockDto {
    let request = tonic::Request::new(BlocksRequest {
        height,
        hash: "".to_string(),
    });
    let cur = get_mempool_client().await.get_block(request).await;
    match cur {
        Ok(t) => {
            let res = t.into_inner();
            tracing::info!("Block: '{:?}' from service", res);
            BlockDto::build_from_msg(res)
        }
        Err(e) => {
            tracing::error!("Error getting block: {:?}", e);
            BlockDto::default()
        }
    }
}

pub async fn get_latest_block() -> BlockDto {
    let request = tonic::Request::new(BlocksRequest {
        height: 0,
        hash: "".to_string(),
    });
    let response = get_mempool_client().await.get_blocks(request).await;
    let mut stream = match response {
        Ok(stream) => stream.into_inner(),
        Err(e) => {
            tracing::error!("Error getting blocks: {:?}", e);
            return BlockDto::default();
        }
    };
    if let Some(res) = stream.message().await.unwrap() {
        let res = BlockDto::build_from_msg(res);
        tracing::debug!("First block = {:?}", res);
        return res;
    }
    BlockDto::default()
}

async fn get_mempool_client() -> BlocksClient<Channel> {
    let client_ref = Arc::clone(&BLOCKS_CLIENT);
    let guard = client_ref.read().await;
    guard.as_ref().unwrap().clone()
}

pub async fn init_mempool_client() {
    let server_url: String = match std::env::var("SERVER_URL") {
        Ok(val) => val,
        Err(_) => "http://127.0.0.1:3088".to_string(),
    };
    let client_ref = Arc::clone(&BLOCKS_CLIENT);
    loop {
        let cli = BlocksClient::connect(server_url.clone()).await;
        match cli {
            Ok(client) => {
                tracing::info!("✅Connection to the gRPC service mempool is successful!");
                let mut guard = client_ref.write().await;
                guard.replace(client);
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

impl BlockDto {
    pub fn build_from_msg(msg: BlockMsg) -> Self {
        let dt = msg.created_at.parse::<DateTime<Utc>>();
        BlockDto {
            id: msg.id,
            height: msg.height,
            version: msg.version,
            timestamp: msg.timestamp,
            tx_count: msg.tx_count,
            size: msg.size,
            weight: msg.weight,
            merkle_root: msg.merkle_root,
            previousblockhash: msg.previousblockhash,
            mediantime: msg.mediantime,
            nonce: msg.nonce,
            bits: msg.bits,
            difficulty: msg.difficulty,
            created_at: Some(dt.unwrap_or_default()),
        }
    }
}

impl BlockMsg {
    pub fn build_from_dto(dto: BlockDto) -> Self {
        BlockMsg {
            id: dto.id,
            height: dto.height,
            version: dto.version,
            timestamp: dto.timestamp,
            tx_count: dto.tx_count,
            size: dto.size,
            weight: dto.weight,
            merkle_root: dto.merkle_root,
            previousblockhash: dto.previousblockhash,
            mediantime: dto.mediantime,
            nonce: dto.nonce,
            bits: dto.bits,
            difficulty: dto.difficulty,
            created_at: dto.created_at.unwrap().to_rfc3339(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, FromRow, Default)]
pub struct BlockDto {
    pub id: String,
    pub height: i64,
    pub version: i64,
    pub timestamp: i64,
    pub tx_count: i64,
    pub size: i64,
    pub weight: i64,
    pub merkle_root: String,
    pub previousblockhash: String,
    pub mediantime: i64,
    pub nonce: i64,
    pub bits: i64,
    pub difficulty: f64,
    pub created_at: Option<DateTime<Utc>>,
}
