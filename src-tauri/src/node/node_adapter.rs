// Copyright 2024. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::node::node_manager::NodeType;
use crate::process_adapter::{HealthStatus, StatusMonitor};
use crate::progress_trackers::progress_stepper::ChanneledStepUpdate;
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use log::{info, warn};
use minotari_node_grpc_client::grpc::{
    BlockHeader, Empty, GetBlocksRequest, GetNetworkStateRequest, Peer, SyncState,
};
use minotari_node_grpc_client::BaseNodeGrpcClient;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tari_core::transactions::tari_amount::MicroMinotari;
use tari_crypto::ristretto::RistrettoPublicKey;
use tari_shutdown::ShutdownSignal;
use tari_utilities::ByteArray;
use tokio::sync::watch;
use tokio::time::timeout;
use serde_json::json;
use tari_common::configuration::Network;
use tauri_plugin_sentry::sentry;
use tauri_plugin_sentry::sentry::protocol::Event;

use crate::network_utils::{get_best_block_from_block_scan, get_block_info_from_block_scan};


const LOG_TARGET: &str = "tari::universe::minotari_node_adapter";

#[async_trait]
pub trait NodeAdapter {
    fn get_grpc_address(&self) -> Option<(String, u16)>;
    fn get_service(&self) -> Option<NodeAdapterService>;
    async fn get_connection_address(&self) -> Result<String, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub(crate) struct NodeAdapterService {
    grpc_address: String,
    required_sync_peers: u32,
}

impl NodeAdapterService {
    pub fn new(grpc_address: String, required_sync_peers: u32) -> Self {
        Self {
            grpc_address,
            required_sync_peers,
        }
    }

    pub async fn get_network_state(&self) -> Result<BaseNodeStatus, NodeStatusMonitorError> {
        let mut client = BaseNodeGrpcClient::connect(self.grpc_address.clone())
            .await
            .map_err(|_| NodeStatusMonitorError::NodeNotStarted)?;

        let res = client
            .get_network_state(GetNetworkStateRequest {})
            .await
            .map_err(|e| NodeStatusMonitorError::UnknownError(e.into()))?;
        let res = res.into_inner();
        let metadata = match res.metadata {
            Some(metadata) => metadata,
            None => {
                return Err(NodeStatusMonitorError::UnknownError(anyhow!(
                    "No metadata found"
                )));
            }
        };

        Ok(BaseNodeStatus {
            sha_network_hashrate: res.sha3x_estimated_hash_rate,
            randomx_network_hashrate: res.randomx_estimated_hash_rate,
            block_reward: MicroMinotari(res.reward),
            block_height: metadata.best_block_height,
            block_time: metadata.timestamp,
            is_synced: res.initial_sync_achieved,
            num_connections: res.num_connections,
        })
    }

    pub async fn get_historical_blocks(
        &self,
        heights: Vec<u64>,
    ) -> Result<Vec<(u64, String)>, Error> {
        let mut client = BaseNodeGrpcClient::connect(self.grpc_address.clone()).await?;

        let mut res = client
            .get_blocks(GetBlocksRequest { heights })
            .await?
            .into_inner();

        let mut blocks: Vec<(u64, String)> = Vec::new();
        while let Some(block) = res.message().await? {
            let BlockHeader { height, hash, .. } = block
                .block
                .clone()
                .expect("Failed to get block data")
                .header
                .expect("Failed to get block header data");
            let hash: String = hash.iter().fold(String::new(), |mut acc, x| {
                write!(acc, "{:02x}", x).expect("Unable to write");
                acc
            });

            blocks.push((height, hash));
        }
        Ok(blocks)
    }

    pub async fn get_identity(&self) -> Result<NodeIdentity, Error> {
        let mut client = BaseNodeGrpcClient::connect(self.grpc_address.clone()).await?;
        println!("asking for identity to {:?}", self.grpc_address.clone());
        let id = client.identify(Empty {}).await?;
        let res = id.into_inner();

        Ok(NodeIdentity {
            public_key: RistrettoPublicKey::from_canonical_bytes(&res.public_key)
                .map_err(|e| anyhow!(e.to_string()))?,
            public_address: res.public_addresses,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn wait_synced(
        &self,
        progress_tracker: Vec<Option<ChanneledStepUpdate>>,
        shutdown_signal: ShutdownSignal,
    ) -> Result<(), NodeStatusMonitorError> {
        let mut client = BaseNodeGrpcClient::connect(self.grpc_address.clone())
            .await
            .map_err(|_e| NodeStatusMonitorError::NodeNotStarted)?;

        loop {
            if shutdown_signal.is_triggered() {
                break Ok(());
            }

            let tip = client
                .get_tip_info(Empty {})
                .await
                .map_err(|e| NodeStatusMonitorError::UnknownError(e.into()))?;
            let sync_progress = client
                .get_sync_progress(Empty {})
                .await
                .map_err(|e| NodeStatusMonitorError::UnknownError(e.into()))?;
            let tip_res = tip.into_inner();
            let sync_progress = sync_progress.into_inner();
            if tip_res.initial_sync_achieved {
                break Ok(());
            }

            let (initial_sync_tracker, header_sync_tracker, block_sync_tracker) =
                match progress_tracker.as_slice() {
                    [initial_sync, header_sync, block_sync] => {
                        (initial_sync, header_sync, block_sync)
                    }
                    _ => {
                        return Err(NodeStatusMonitorError::UnknownError(anyhow!(
                            "Progress tracker not set up correctly"
                        )));
                    }
                };

            if sync_progress.state == SyncState::Startup as i32 {
                let mut progress_params: HashMap<String, String> = HashMap::new();
                let percentage = sync_progress.initial_connected_peers as f64
                    / f64::from(self.required_sync_peers);
                progress_params.insert(
                    "initial_connected_peers".to_string(),
                    sync_progress.initial_connected_peers.to_string(),
                );
                progress_params.insert(
                    "required_peers".to_string(),
                    self.required_sync_peers.to_string(),
                );
                if let Some(tracker) = initial_sync_tracker {
                    tracker.send_update(progress_params, percentage).await;
                }
            } else if sync_progress.state == SyncState::Header as i32 {
                let mut progress_params: HashMap<String, String> = HashMap::new();
                let percentage =
                    sync_progress.local_height as f64 / sync_progress.tip_height as f64;
                progress_params.insert(
                    "local_header_height".to_string(),
                    sync_progress.local_height.to_string(),
                );
                progress_params.insert(
                    "tip_header_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );
                progress_params.insert("local_block_height".to_string(), "0".to_string());
                progress_params.insert(
                    "tip_block_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );
                // Keep these fields for old translations that have not been updated
                progress_params.insert(
                    "local_height".to_string(),
                    sync_progress.local_height.to_string(),
                );
                progress_params.insert(
                    "tip_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );
                if let Some(tracker) = header_sync_tracker {
                    tracker.send_update(progress_params, percentage).await;
                }
            } else if sync_progress.state == SyncState::Block as i32 {
                let mut progress_params: HashMap<String, String> = HashMap::new();
                let percentage =
                    sync_progress.local_height as f64 / sync_progress.tip_height as f64;
                progress_params.insert(
                    "local_header_height".to_string(),
                    sync_progress.local_height.to_string(),
                );
                progress_params.insert(
                    "tip_header_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );
                progress_params.insert(
                    "local_block_height".to_string(),
                    sync_progress.local_height.to_string(),
                );
                progress_params.insert(
                    "tip_block_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );
                // Keep these fields for old translations that have not been updated
                progress_params.insert(
                    "local_height".to_string(),
                    sync_progress.local_height.to_string(),
                );
                progress_params.insert(
                    "tip_height".to_string(),
                    sync_progress.tip_height.to_string(),
                );

                if let Some(tracker) = block_sync_tracker {
                    tracker.send_update(progress_params, percentage).await;
                }
            } else {
                // do nothing
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    pub async fn list_connected_peers(&self) -> Result<Vec<Peer>, Error> {
        let mut client = BaseNodeGrpcClient::connect(self.grpc_address.clone()).await?;
        let connected_peers = client.list_connected_peers(Empty {}).await?;
        Ok(connected_peers.into_inner().connected_peers)
    }

    pub async fn check_if_is_orphan_chain(
        &self,
        report_to_sentry: bool,
    ) -> Result<bool, anyhow::Error> {
        let BaseNodeStatus {
            is_synced,
            block_height: local_tip,
            ..
        } = self.get_network_state().await?;
        if !is_synced {
            info!(target: LOG_TARGET, "Node is not synced, skipping orphan chain check");
            return Ok(false);
        }

        let network = Network::get_current_or_user_setting_or_default();
        let block_scan_tip = get_best_block_from_block_scan(network).await?;
        let heights: Vec<u64> = vec![
            block_scan_tip.saturating_sub(50),
            block_scan_tip.saturating_sub(100),
            block_scan_tip.saturating_sub(200),
        ];
        let mut block_scan_blocks: Vec<(u64, String)> = vec![];

        for height in &heights {
            let block_scan_block = get_block_info_from_block_scan(network, height).await?;
            block_scan_blocks.push(block_scan_block);
        }

        let local_blocks = self.get_historical_blocks(heights).await?;
        for block_scan_block in &block_scan_blocks {
            if !local_blocks
                .iter()
                .any(|local_block| block_scan_block.1 == local_block.1)
            {
                if report_to_sentry {
                    let error_msg = "Orphan chain detected".to_string();
                    let extra = vec![
                        (
                            "block_scan_block_height".to_string(),
                            json!(block_scan_block.0.to_string()),
                        ),
                        (
                            "block_scan_block_hash".to_string(),
                            json!(block_scan_block.1.clone()),
                        ),
                        (
                            "block_scan_tip_height".to_string(),
                            json!(block_scan_tip.to_string()),
                        ),
                        ("local_tip_height".to_string(), json!(local_tip.to_string())),
                    ];
                    sentry::capture_event(Event {
                        message: Some(error_msg),
                        level: sentry::Level::Error,
                        culprit: Some("orphan-chain".to_string()),
                        extra: extra.into_iter().collect(),
                        ..Default::default()
                    });
                }
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[derive(Clone)]
pub(crate) struct NodeStatusMonitor {
    node_type: NodeType,
    node_client: NodeAdapterService,
    status_broadcast: watch::Sender<BaseNodeStatus>,
    #[allow(dead_code)]
    last_block_time: Arc<AtomicU64>,
}

impl NodeStatusMonitor {
    pub fn new(
        node_type: NodeType,
        node_client: NodeAdapterService,
        status_broadcast: watch::Sender<BaseNodeStatus>,
        last_block_time: Arc<AtomicU64>,
    ) -> Self {
        Self {
            node_type,
            node_client,
            status_broadcast,
            last_block_time,
        }
    }
}

#[async_trait]
impl StatusMonitor for NodeStatusMonitor {
    async fn check_health(&self, _uptime: Duration) -> HealthStatus {
        let duration = std::time::Duration::from_secs(5);
        match timeout(duration, self.node_client.get_network_state()).await {
            Ok(res) => match res {
                Ok(status) => {
                    let _res = self.status_broadcast.send(status.clone());
                    // Remote Node always returns 0 connections
                    if status.num_connections == 0 && self.node_type != NodeType::Remote {
                        warn!(
                            "{:?} Node Health Check Warning: No connections",
                            self.node_type
                        );
                        return HealthStatus::Warning;
                    }
                    // if self
                    //     .last_block_time
                    //     .load(std::sync::atomic::Ordering::SeqCst)
                    //     == status.block_time
                    // {
                    //     if uptime.as_secs() > 1200
                    //         && EpochTime::now()
                    //             .checked_sub(EpochTime::from_secs_since_epoch(status.block_time))
                    //             .unwrap_or(EpochTime::from(0))
                    //             .as_u64()
                    //             > 1200
                    //     {
                    //         warn!(target: LOG_TARGET, "Base node height has not changed in twenty minutes");
                    //         return HealthStatus::Warning;
                    //     }
                    // } else {
                    //     self.last_block_time
                    //         .store(status.block_time, std::sync::atomic::Ordering::SeqCst);
                    // }
                    HealthStatus::Healthy
                }
                Err(e) => {
                    warn!(
                        "{:?} Node Health Check Error: checking base node status: {:?}",
                        self.node_type, e
                    );
                    HealthStatus::Unhealthy
                }
            },
            Err(e) => {
                warn!("{:?} Node Health Check Error. {:?}", self.node_type, e);
                match self.node_client.get_identity().await {
                    Ok(_) => match self.node_client.get_identity().await {
                        Ok(identity) => {
                            info!(target: LOG_TARGET, "{:?} Node identity: {:?}", self.node_type, identity);
                            return HealthStatus::Healthy;
                        }
                        Err(e) => {
                            warn!(
                                "{:?} Node Health Check Error: checking base node identity: {:?}",
                                self.node_type, e
                            );
                            return HealthStatus::Unhealthy;
                        }
                    },
                    Err(e) => {
                        warn!(
                            "{:?} Node Health Check Error: checking base node identity: {:?}",
                            self.node_type, e
                        );
                        return HealthStatus::Unhealthy;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NodeIdentity {
    pub public_key: RistrettoPublicKey,
    pub public_address: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct BaseNodeStatus {
    pub sha_network_hashrate: u64,
    pub randomx_network_hashrate: u64,
    pub block_reward: MicroMinotari,
    pub block_height: u64,
    pub block_time: u64,
    pub is_synced: bool,
    pub num_connections: u64,
}

impl Default for BaseNodeStatus {
    fn default() -> Self {
        Self {
            sha_network_hashrate: 0,
            randomx_network_hashrate: 0,
            block_reward: MicroMinotari(0),
            block_height: 0,
            block_time: 0,
            is_synced: false,
            num_connections: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NodeStatusMonitorError {
    #[error("Unknown error: {0}")]
    UnknownError(#[from] anyhow::Error),
    #[error("Node not started")]
    NodeNotStarted,
}
