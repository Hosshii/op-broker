use crate::config::BrokerConfig;
use protocol::{
    SecretId, pb::ReadSecretRequest, pb::ReadSecretResponse,
    pb::broker_service_server::BrokerService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct BrokerRpcService {
    config: Arc<BrokerConfig>,
}

impl BrokerRpcService {
    pub fn new(config: Arc<BrokerConfig>) -> Self {
        Self { config }
    }
}

impl Clone for BrokerRpcService {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
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
        let id = SecretId::parse(&request.id)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let Some(item) = self.config.items.get(&id) else {
            return Err(Status::not_found("secret id not permitted"));
        };
        tracing::info!(id = %id, op_path = %item.op_path, "serving read request");

        // TODO: Invoke `op read` and return the secret value trimmed.
        let response = ReadSecretResponse {
            value: format!("mock-secret-for:{}", item.op_path),
        };
        Ok(Response::new(response))
    }
}
