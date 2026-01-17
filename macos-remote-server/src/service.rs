use macos_remote_protocol::pb::{
    NotifyRequest, NotifyResponse, OpReadRequest, OpReadResponse,
    mac_os_remote_service_server::MacOsRemoteService,
};
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::notify::Notifier;
use crate::op_client::OpClient;

pub struct MacOsRemoteServiceImpl {
    notifier: Option<Notifier>,
    op_client: Option<OpClient>,
}

impl MacOsRemoteServiceImpl {
    pub fn new() -> Self {
        let notifier = match Notifier::discover() {
            Ok(n) => {
                info!("terminal-notifier discovered");
                Some(n)
            }
            Err(e) => {
                error!("terminal-notifier not available: {e}");
                None
            }
        };

        let op_client = match OpClient::discover() {
            Ok(c) => {
                info!("op CLI discovered");
                Some(c)
            }
            Err(e) => {
                error!("op CLI not available: {e}");
                None
            }
        };

        Self {
            notifier,
            op_client,
        }
    }
}

#[tonic::async_trait]
impl MacOsRemoteService for MacOsRemoteServiceImpl {
    async fn notify(
        &self,
        request: Request<NotifyRequest>,
    ) -> Result<Response<NotifyResponse>, Status> {
        let req = request.into_inner();
        info!(title = %req.title, message = %req.message, "notify request");

        let notifier = self
            .notifier
            .as_ref()
            .ok_or_else(|| Status::unavailable("terminal-notifier not available"))?;

        match notifier.notify(&req.title, &req.message).await {
            Ok(()) => Ok(Response::new(NotifyResponse {
                success: true,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(NotifyResponse {
                success: false,
                error: e.to_string(),
            })),
        }
    }

    async fn op_read(
        &self,
        request: Request<OpReadRequest>,
    ) -> Result<Response<OpReadResponse>, Status> {
        let req = request.into_inner();
        info!(reference = %req.reference, "op_read request");

        let op_client = self
            .op_client
            .as_ref()
            .ok_or_else(|| Status::unavailable("op CLI not available"))?;

        match op_client.read(&req.reference).await {
            Ok(value) => Ok(Response::new(OpReadResponse { value })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
