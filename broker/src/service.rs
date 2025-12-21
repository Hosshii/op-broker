use crate::{
    config::BrokerConfig,
    op_client::{OpClient, OpError},
};
use protocol::{
    OpSecretReference, pb::ReadSecretRequest, pb::ReadSecretResponse,
    pb::broker_service_server::BrokerService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct BrokerRpcService {
    config: Arc<BrokerConfig>,
    op_client: Arc<OpClient>,
}

impl BrokerRpcService {
    pub fn new(config: Arc<BrokerConfig>, op_client: Arc<OpClient>) -> Self {
        Self { config, op_client }
    }
}

impl Clone for BrokerRpcService {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            op_client: Arc::clone(&self.op_client),
        }
    }
}

#[tonic::async_trait]
impl BrokerService for BrokerRpcService {
    async fn read_secret(
        &self,
        request: Request<ReadSecretRequest>,
    ) -> Result<Response<ReadSecretResponse>, Status> {
        let request = request.into_inner();
        let reference = OpSecretReference::parse(&request.secret_reference)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let Some(item) = self.config.resolve(&reference) else {
            return Err(Status::not_found("secret reference not permitted"));
        };
        tracing::info!(reference = %item.reference, "serving read request");

        match self.op_client.read(item.reference).await {
            Ok(value) => Ok(Response::new(ReadSecretResponse { value })),
            Err(err) => {
                tracing::error!(
                    reference = %item.reference,
                    error = %err,
                    "failed to read secret"
                );
                Err(map_op_error(err))
            }
        }
    }
}

fn map_op_error(err: OpError) -> Status {
    match err {
        OpError::ExecutableNotFound => Status::failed_precondition("op CLI not found in PATH"),
        OpError::Timeout => Status::deadline_exceeded("op CLI timed out"),
        OpError::Io(_) | OpError::CommandFailed { .. } => {
            Status::internal("op CLI returned an error")
        }
        OpError::InvalidUtf8 | OpError::EmptyResponse => {
            Status::internal("op CLI response was invalid")
        }
    }
}
