use crate::mm_proxy_manager::MmProxyManager;
use crate::xmrig::http_api::XmrigHttpApiClient;
use crate::xmrig_adapter::{XmrigAdapter, XmrigNodeConnection};
use crate::{
    CpuMinerConfig, CpuMinerConnection, CpuMinerConnectionStatus, CpuMinerStatus, MiningMode,
};
use log::warn;
use std::path::PathBuf;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tari_core::transactions::tari_amount::MicroMinotari;
use tari_shutdown::{Shutdown, ShutdownSignal};
use tauri::async_runtime::JoinHandle;
use tokio::select;
use tokio::time::MissedTickBehavior;

const RANDOMX_BLOCKS_PER_DAY: u64 = 350;
const LOG_TARGET: &str = "tari::universe::cpu_miner";
pub enum CpuMinerEvent {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

pub(crate) struct CpuMiner {
    watcher_task: Option<JoinHandle<Result<(), anyhow::Error>>>,
    miner_shutdown: Shutdown,
    api_client: Option<XmrigHttpApiClient>,
}

impl CpuMiner {
    pub fn new() -> Self {
        Self {
            watcher_task: None,
            miner_shutdown: Shutdown::new(),
            api_client: None,
        }
    }

    pub async fn start(
        &mut self,
        mut app_shutdown: ShutdownSignal,
        cpu_miner_config: &CpuMinerConfig,
        local_mm_proxy: &MmProxyManager,
        base_path: PathBuf,
        cache_dir: PathBuf,
        log_dir: PathBuf,
        window: tauri::Window,
        mode: MiningMode,
    ) -> Result<(), anyhow::Error> {
        if self.watcher_task.is_some() {
            warn!(target: LOG_TARGET, "Tried to start mining twice");
            return Ok(());
        }
        self.miner_shutdown = Shutdown::new();
        let mut inner_shutdown = self.miner_shutdown.to_signal();

        let xmrig_node_connection = match cpu_miner_config.node_connection {
            CpuMinerConnection::BuiltInProxy => {
                local_mm_proxy
                    .start(
                        app_shutdown.clone(),
                        base_path.clone(),
                        cpu_miner_config.tari_address.clone(),
                        window.clone(),
                    )
                    .await?;
                local_mm_proxy.wait_ready().await?;
                XmrigNodeConnection::LocalMmproxy {
                    host_name: "127.0.0.1".to_string(),
                    // port: local_mm_proxy.try_get_listening_port().await?
                    // TODO: Replace with actual port
                    port: 18081,
                }
            }
        };
        let cpu_max_percentage = match mode {
            MiningMode::Eco => 30,
            MiningMode::Ludicrous => 100,
        };
        let xmrig = XmrigAdapter::new(xmrig_node_connection, "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A".to_string()  );
        let (mut _rx, mut xmrig_child, client) =
            xmrig.spawn(cache_dir, log_dir, base_path, window.clone(), cpu_max_percentage)?;
        self.api_client = Some(client);

        self.watcher_task = Some(tauri::async_runtime::spawn(async move {
            println!("Starting process");
            let mut watch_timer = tokio::time::interval(tokio::time::Duration::from_secs(1));
            watch_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
            // read events such as stdout
            loop {
                select! {
                              _ = watch_timer.tick() => {
                                    println!("watching");
                                    if xmrig_child.ping().expect("idk") {
                                       println!("xmrig is running");
                                    } else {
                                       println!("xmrig is not running");
                                       match xmrig_child.stop().await {
                                           Ok(()) => {
                                              println!("xmrig exited successfully");
                                           }
                                           Err(e) => {
                                              println!("xmrig exited with error: {}", e);
                                           }
                                       }
                                       break;
                                    }
                              },
                                //   event = rx.recv() => {
                                //     if let Some(event) = event {
                                //
                                //   // if let CommandEvent::Stdout(line) = event {
                                //   //    window
                                //   //   .emit("message", Some(format!("'{}'", line)))
                                //   //   .expect("failed to emit event");
                                // // write to stdin
                                // //child.write("message from Rust\n".as_bytes()).unwrap();
                                //
                                //    }
                                // else {
                                //  break;
                                // }
                          //         },
                //
                            _ = inner_shutdown.wait() => {
                                xmrig_child.stop().await?;
                                break;
                            },
                            _ = app_shutdown.wait() => {
                                xmrig_child.stop().await?;
                                break;
                            }
                        }
            }
            Ok(())
        }));
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), anyhow::Error> {
        println!("Triggering shutdown");
        self.miner_shutdown.trigger();
        self.api_client = None;
        if let Some(task) = self.watcher_task.take() {
            task.await?;
            println!("Task finished");
        }
        // TODO: This doesn't seem to be called

        Ok(())
    }

    pub async fn status(
        &self,
        network_hash_rate: u64,
        block_reward: MicroMinotari,
    ) -> Result<CpuMinerStatus, anyhow::Error> {
        let mut s =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

        // Wait a bit because CPU usage is based on diff.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        // Refresh CPUs again.
        s.refresh_cpu_all();

        let cpu_brand = s.cpus().get(0).map(|cpu| cpu.brand()).unwrap_or("Unknown");

        let cpu_usage = s.global_cpu_usage() as u32;

        match &self.api_client {
            Some(client) => {
                let xmrig_status = client.summary().await?;
                let hash_rate = xmrig_status.hashrate.total[0].unwrap_or_default();
                dbg!(hash_rate, network_hash_rate, block_reward);
                let estimated_earnings = (block_reward.as_u64() as f64
                    * (hash_rate / network_hash_rate as f64 * RANDOMX_BLOCKS_PER_DAY as f64))
                    as u64;
                // Can't be more than the max reward for a day
                let estimated_earnings = std::cmp::min(
                    estimated_earnings,
                    block_reward.as_u64() * RANDOMX_BLOCKS_PER_DAY,
                );

                Ok(CpuMinerStatus {
                    is_mining: xmrig_status.hashrate.total.len() > 0
                        && xmrig_status.hashrate.total[0].is_some()
                        && xmrig_status.hashrate.total[0].unwrap() > 0.0,
                    hash_rate,
                    cpu_usage: cpu_usage as u32,
                    cpu_brand: cpu_brand.to_string(),
                    estimated_earnings: MicroMinotari(estimated_earnings).as_u64(),
                    connection: CpuMinerConnectionStatus {
                        is_connected: xmrig_status.connection.uptime > 0,
                        // error: if xmrig_status.connection.error_log.is_empty() {
                        //     None
                        // } else {
                        //     Some(xmrig_status.connection.error_log.join(";"))
                        // },
                    },
                })
            }
            None => Ok(CpuMinerStatus {
                is_mining: false,
                hash_rate: 0.0,
                cpu_usage: cpu_usage as u32,
                cpu_brand: cpu_brand.to_string(),
                estimated_earnings: 0,
                connection: CpuMinerConnectionStatus {
                    is_connected: false,
                    // error: None,
                },
            }),
        }
    }
}
