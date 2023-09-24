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
    client: SyncerClient<Channel>,
    runtime: Runtime,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for Client {}

#[godot_api]
impl Client {
    fn connect(base: Base<Node>, dst: GodotString) -> Result<Self, tonic::transport::Error> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let client = runtime.block_on(SyncerClient::connect(dst.to_string()))?;

        Ok(Self {
            client,
            runtime,
            base,
        })
    }
}
