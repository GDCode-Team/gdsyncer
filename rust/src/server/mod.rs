use godot::prelude::*;
use jwt_simple::prelude::*;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use syncer::{
    syncer_server::{Syncer, SyncerServer},
    JoinReply, JoinRequest,
};
use tokio::{
    runtime::{Builder, Runtime},
    sync::{mpsc, Mutex},
};
use tonic::{Code, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};
use uuid::Uuid;

use crate::utils::VERSION;

pub mod syncer {
    tonic::include_proto!("syncer");
}

pub struct User {
    pub name: String,
    pub color: Color,
    key: HS256Key,
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize)]
struct UserTokenData {
    uuid: String,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            name,
            color: Color::from_rgb(0.5, 0.5, 0.5),
            key: HS256Key::generate(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn generate_token(&self) -> String {
        let user_data = UserTokenData {
            uuid: self.uuid.to_string(),
        };

        let claims = Claims::with_custom_claims(user_data, Duration::from_hours(2));
        self.key.authenticate(claims).unwrap()
    }
}

pub struct ServerBasic {
    users: Mutex<HashMap<Uuid, User>>,
    password: String,
}

impl ServerBasic {
    pub fn new(password: String) -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            password,
        }
    }
}

#[tonic::async_trait]
impl Syncer for ServerBasic {
    async fn join(&self, request: Request<JoinRequest>) -> Result<Response<JoinReply>, Status> {
        let mut users = self.users.lock().await;

        let inner_request = request.into_inner();

        let mut err_details = ErrorDetails::new();

        if inner_request.version != VERSION {
            err_details.add_bad_request_violation(
                "version",
                "Your version does not match the server ones.",
            );
        };

        if inner_request.password != self.password {
            err_details
                .add_bad_request_violation("password", "The password you entered is invalid.");
        };

        if err_details.has_bad_request_violations() {
            let status = Status::with_error_details(
                Code::InvalidArgument,
                "Your request contains invalid arguments.",
                err_details,
            );

            return Err(status);
        };

        let user = User::new(inner_request.name);

        let token = user.generate_token();
        let color = user.color;

        users.insert(user.uuid, user);

        Ok(Response::new(JoinReply {
            token,
            color: color.to_string(),
        }))
    }
}

#[derive(Debug, GodotClass)]
#[class(tool, base=Node)]
pub struct Server {
    pub address: SocketAddr,
    pub password: String,
    runtime: Runtime,
    pub running: Arc<Mutex<bool>>,
    tx: mpsc::Sender<bool>,
    rx: Mutex<mpsc::Receiver<bool>>,
    #[base]
    base: Base<Node>,
}

#[godot_api]
impl NodeVirtual for Server {
    fn exit_tree(&mut self) {
        self.shutdown_sync();
    }
}

#[godot_api]
impl Server {
    #[signal]
    fn change_state();

    pub fn new(base: Base<Node>) -> Self {
        let (tx, rx) = mpsc::channel(8);

        Self {
            address: "127.0.0.1:8008".parse().unwrap(),
            password: "123".to_string(),
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
            running: Arc::new(Mutex::new(false)),
            tx,
            rx: Mutex::new(rx),
            base,
        }
    }

    #[func]
    pub fn start_sync(&self) {
        self.runtime.handle().block_on(self.start());
    }

    #[func]
    pub fn shutdown_sync(&self) {
        self.runtime.handle().block_on(self.shutdown());
    }

    #[func]
    pub fn start_or_shutdown_sync(&self) {
        self.runtime.handle().block_on(self.start_or_shutdown());
    }

    pub async fn start_or_shutdown(&self) -> Result<(), tonic::transport::Error> {
        if self.is_running().await {
            self.shutdown().await;

            return Ok(());
        }

        self.start().await
    }

    pub async fn start(&self) -> Result<(), tonic::transport::Error> {
        if self.is_running().await {
            return Ok(());
        }

        godot_print!("[GDSyncer] Starting up server...");

        self.set_running(true).await;

        let server_basic = ServerBasic::new(self.password.clone());

        let res = tonic::transport::Server::builder()
            .add_service(SyncerServer::new(server_basic))
            .serve_with_shutdown(self.address, self.wait_for_shutdown())
            .await;

        self.set_running(false).await;

        if let Err(error) = res {
            Err(error.into())
        } else {
            Ok(())
        }
    }

    pub async fn shutdown(&self) {
        if !self.is_running().await {
            return;
        }

        godot_print!("[GDSyncer] Shutting down server...");

        self.set_running(false).await;

        godot_print!("[GDSyncer] Server has been successfully shut down.")
    }

    async fn wait_for_shutdown(&self) {
        let mut rx = self.rx.lock().await;

        while let Some(running) = rx.recv().await {
            if running {
                return;
            }
        }
    }

    async fn set_running(&self, value: bool) {
        if self.is_running().await == value {
            return;
        }

        *(self.running.lock().await) = value;

        self.tx.send(value).await.unwrap();
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    #[func]
    fn is_running_sync(&self) -> bool {
        self.runtime.handle().block_on(self.is_running())
    }
}
