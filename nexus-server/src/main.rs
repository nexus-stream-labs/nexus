
use clap::Parser;
use config::Config;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::{info, error, Level};
use tracing_subscriber;
use uuid::Uuid;

use nexus_broker::{Broker, MessageBroker, Topic};
use nexus_storage::{RocksDbStorage, StorageEngine};
use nexus_consensus::{RaftConsensus, ConsensusEngine};
use nexus_streaming::{WindowedProcessor, StreamProcessor};
use nexus_gateway::create_router;

#[derive(Parser)]
#[command(name = "nexus-server")]
#[command(about = "Nexus distributed event streaming platform")]
struct Cli {
    #[arg(long, default_value = "config/single-node.toml")]
    config: String,
    
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[arg(long, default_value = "0.0.0.0:9090")]
    bind_address: String,
}

#[derive(Debug)]
struct NexusServer {
    config: Config,
    broker: Broker,
    storage: RocksDbStorage,
    consensus: RaftConsensus,
    streaming_processor: WindowedProcessor,
}

impl NexusServer {
    async fn new(config_path: &str) -> anyhow::Result<Self> {
        // Load configuration
        let config = Config::builder()
            .add_source(config::File::with_name(config_path))
            .build()?;
        
        // Initialize storage
        let data_path = config.get_string("storage.data_path")
            .unwrap_or_else(|_| "./data/storage".to_string());
        let storage = RocksDbStorage::new(&data_path)?;
        
        // Initialize consensus
        let node_id = Uuid::new_v4();
        let consensus = RaftConsensus::new(node_id);
        
        // Initialize broker
        let broker = Broker::new();
        
        // Initialize streaming processor
        let window_duration = std::time::Duration::from_secs(60);
        let streaming_processor = WindowedProcessor::new(window_duration);
        
        Ok(Self {
            config,
            broker,
            storage,
            consensus,
            streaming_processor,
        })
    }
    
    async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting Nexus Event Streaming Platform");
        
        // Create default topic for testing
        let default_topic = Topic {
            name: "default".to_string(),
            partitions: 1,
            replication_factor: 1,
            retention_ms: 604800000, // 7 days
        };
        
        self.broker.create_topic(default_topic).await?;
        info!("Created default topic");
        
        // Start API gateway
        let gateway_port = self.config.get_int("gateway.rest_port")
            .unwrap_or(8080) as u16;
        
        let app = create_router();
        let gateway_addr = format!("0.0.0.0:{}", gateway_port);
        let listener = TcpListener::bind(&gateway_addr).await?;
        
        info!("API Gateway listening on {}", gateway_addr);
        
        // Start the gateway in a background task
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Gateway server error: {}", e);
            }
        });
        
        Ok(())
    }
    
    async fn run(&mut self) -> anyhow::Result<()> {
        self.start().await?;
        
        info!("Nexus server started successfully");
        
        // Keep the server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            // Health check
            if self.consensus.is_leader().await {
                info!("Health check: Node is leader");
            } else {
                info!("Health check: Node is follower");
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    
    // Create and start server
    let mut server = NexusServer::new(&cli.config).await?;
    server.run().await
}
