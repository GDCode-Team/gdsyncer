use std::sync::Arc;

use godot::prelude::*;
use syncer::syncer_client::SyncerClient;
use tokio::{
    runtime::{Builder, Runtime},
    sync::Mutex,
};
use tonic::transport::Channel;

pub mod syncer {
    tonic::include_proto!("syncer");
}

struct Client {
    client: Option<SyncerClient<Channel>>,
}

impl Client {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub async fn connect(&mut self, dst: String) -> Result<(), tonic::transport::Error> {
        self.client = Some(SyncerClient::connect(dst).await?);

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

#[derive(GodotClass)]
#[class(tool, base=Node)]
pub struct ClientWrapper {
    client: Arc<Mutex<Client>>,
    runtime: Runtime,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl ClientWrapper {
    pub fn new(base: Base<Node>) -> Self {
        let client = Arc::new(Mutex::new(Client::new()));

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        Self {
            client,
            runtime,
            base,
        }
    }

    pub fn connect_sync(&self, dst: String) {
        let client = self.client.clone();

        self.runtime.handle().spawn(async move {
            let mut client = client.lock().await;
            if let Err(err) = client.connect(dst).await {
                godot_error!("An error appeared when trying to connect: {:?}", err);
            } else {
                godot_print!("Client connected successfully.")
            }
        });
    }

    pub fn is_connected(&self) -> bool {
        self.client.blocking_lock().is_connected()
    }
}
