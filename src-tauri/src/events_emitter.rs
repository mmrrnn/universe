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

use log::error;
use serde::Serialize;
use tari_common_types::tari_address::TariAddress;
use tauri::{AppHandle, Emitter};

use crate::{
    commands::CpuMinerStatus,
    hardware::hardware_status_monitor::PublicDeviceProperties,
    wallet_adapter::{TransactionInfo, WalletBalance},
    BaseNodeStatus, GpuMinerStatus,
};

const LOG_TARGET: &str = "tari::universe::events_emitter";

#[derive(Debug, Serialize, Clone)]
pub enum FrontendEventType {
    WalletAddressUpdate,
    WalletBalanceUpdate,
    BaseNodeUpdate,
    GpuDevicesUpdate,
    CpuMiningUpdate,
    GpuMiningUpdate,
    ConnectedPeersUpdate,
    NewBlockHeight,
}

#[derive(Clone, Debug, Serialize)]
struct FrontendEvent<T> {
    event_type: FrontendEventType,
    payload: T,
}

#[derive(Clone, Debug, Serialize)]
struct WalletAddressUpdatePayload {
    tari_address_base58: String,
    tari_address_emoji: String,
}

#[derive(Clone, Debug, Serialize)]
struct NewBlockHeightPayload {
    block_height: u64,
    coinbase_transaction: Option<TransactionInfo>,
    balance: WalletBalance,
}

pub(crate) struct EventsEmitter;

impl EventsEmitter {
    pub async fn emit_wallet_address_update(app_handle: &AppHandle, wallet_address: TariAddress) {
        let event = FrontendEvent {
            event_type: FrontendEventType::WalletAddressUpdate,
            payload: WalletAddressUpdatePayload {
                tari_address_base58: wallet_address.to_base58(),
                tari_address_emoji: wallet_address.to_emoji_string(),
            },
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit WalletAddressUpdate event: {:?}", e);
        }
    }

    pub async fn emit_wallet_balance_update(app_handle: &AppHandle, balance: WalletBalance) {
        let event = FrontendEvent {
            event_type: FrontendEventType::WalletBalanceUpdate,
            payload: balance,
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit WalletBalanceUpdate event: {:?}", e);
        }
    }

    pub async fn emit_base_node_update(app_handle: &AppHandle, status: BaseNodeStatus) {
        let event = FrontendEvent {
            event_type: FrontendEventType::BaseNodeUpdate,
            payload: status,
        };

        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit BaseNodeUpdate event: {:?}", e);
        }
    }

    pub async fn emit_gpu_devices_update(
        app_handle: &AppHandle,
        gpu_public_devices: Vec<PublicDeviceProperties>,
    ) {
        let event = FrontendEvent {
            event_type: FrontendEventType::GpuDevicesUpdate,
            payload: gpu_public_devices,
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit GpuDevicesUpdate event: {:?}", e);
        }
    }

    pub async fn emit_cpu_mining_update(app_handle: &AppHandle, status: CpuMinerStatus) {
        let event = FrontendEvent {
            event_type: FrontendEventType::CpuMiningUpdate,
            payload: status,
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit CpuMiningUpdate event: {:?}", e);
        }
    }

    pub async fn emit_gpu_mining_update(app_handle: &AppHandle, status: GpuMinerStatus) {
        let event = FrontendEvent {
            event_type: FrontendEventType::GpuMiningUpdate,
            payload: status,
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit GpuMiningUpdate event: {:?}", e);
        }
    }

    pub async fn emit_connected_peers_update(app_handle: &AppHandle, connected_peers: Vec<String>) {
        let event = FrontendEvent {
            event_type: FrontendEventType::ConnectedPeersUpdate,
            payload: connected_peers,
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit ConnectedPeersUpdate event: {:?}", e);
        }
    }

    pub async fn emit_new_block_mined(
        app_handle: &AppHandle,
        block_height: u64,
        coinbase_transaction: Option<TransactionInfo>,
        balance: WalletBalance
    ) {
        let event = FrontendEvent {
            event_type: FrontendEventType::NewBlockHeight,
            payload: NewBlockHeightPayload {
                block_height,
                coinbase_transaction,
                balance,
            },
        };
        if let Err(e) = app_handle.emit("frontend_event", event) {
            error!(target: LOG_TARGET, "Failed to emit NewBlockHeight event: {:?}", e);
        }
    }
}
