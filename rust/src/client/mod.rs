use godot::prelude::*;
use syncer::syncer_client::SyncerClient;
use tokio::runtime::{Builder, Runtime};
use tonic::transport::Channel;

pub mod syncer {
    tonic::include_proto!("syncer");
}

#[derive(GodotClass)]
#[class(tool, base=Node)]
pub struct Client {
    client: Option<SyncerClient<Channel>>,
    runtime: Runtime,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for Client {}

#[godot_api]
impl Client {
    pub fn new(base: Base<Node>) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        Self {
            client: None,
            runtime,
            base,
        }
    }

    pub fn connect_to(&mut self, dst: String) -> Result<(), tonic::transport::Error> {
        self.client = Some(
            self.runtime
                .block_on(SyncerClient::connect(dst.to_string()))?,
        );

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}
