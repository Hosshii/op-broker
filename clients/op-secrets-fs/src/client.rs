use anyhow::{Context, Result};
use hyper_util::rt::TokioIo;
use protocol::pb::{ReadSecretRequest, broker_service_client::BrokerServiceClient};
use std::{path::PathBuf, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{net::UnixStream, time};
use tonic::{
    Code, Request,
    transport::{Channel, Endpoint},
};
use tower::service_fn;

pub struct OpBrokerClient {
    inner: BrokerServiceClient<Channel>,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("secret request denied")]
    PermissionDenied,
    #[error("secret reference not found")]
    NotFound,
    #[error("invalid request")]
    InvalidRequest,
    #[error("request timed out")]
    Timeout,
    #[error("broker unavailable")]
    Unavailable,
    #[error("internal client error")]
    Internal,
}

impl OpBrokerClient {
    pub async fn connect(socket_path: PathBuf) -> Result<Self> {
        let channel = connect_via_uds(socket_path)
            .await
            .context("failed to connect to broker over unix socket")?;
        Ok(Self {
            inner: BrokerServiceClient::new(channel),
        })
    }

    pub async fn read_secret(
        &mut self,
        reference: String,
        timeout: Duration,
    ) -> Result<Vec<u8>, ClientError> {
        let request = Request::new(ReadSecretRequest {
            secret_reference: reference,
            nonce: String::new(),
        });
        let future = self.inner.read_secret(request);
        let response = time::timeout(timeout, future)
            .await
            .map_err(|_| ClientError::Timeout)?
            .map_err(ClientError::from)?;
        Ok(response.into_inner().value.into_bytes())
    }
}

async fn connect_via_uds(path: PathBuf) -> Result<Channel, tonic::transport::Error> {
    let endpoint = Endpoint::from_static("http://[::]:50051");
    let path = Arc::new(path);
    endpoint
        .connect_with_connector(service_fn(move |_| {
            let path = path.clone();
            async move {
                let stream = UnixStream::connect(path.as_ref()).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
}

impl From<tonic::Status> for ClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            Code::InvalidArgument => ClientError::InvalidRequest,
            Code::NotFound => ClientError::NotFound,
            Code::PermissionDenied | Code::Unauthenticated => ClientError::PermissionDenied,
            Code::DeadlineExceeded => ClientError::Timeout,
            Code::FailedPrecondition | Code::Unavailable | Code::ResourceExhausted => {
                ClientError::Unavailable
            }
            _ => ClientError::Internal,
        }
    }
}
