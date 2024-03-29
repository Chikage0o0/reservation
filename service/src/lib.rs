use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse, ListenRequest,
    QueryRequest, Reservation, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use anyhow::Result;
use reservation::ReservationManager;
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub struct RsvpService {
    _manager: ReservationManager,
}

impl RsvpService {
    pub async fn new(config: abi::config::Config) -> Result<Self> {
        let manager = ReservationManager::new(config.db).await?;
        Ok(Self { _manager: manager })
    }
}

#[tonic::async_trait]
impl ReservationService for RsvpService {
    async fn reserve(
        &self,
        _request: Request<ReserveRequest>,
    ) -> Result<Response<ReserveResponse>, Status> {
        todo!()
    }
    async fn confirm(
        &self,
        _request: Request<ConfirmRequest>,
    ) -> Result<Response<ConfirmResponse>, Status> {
        todo!()
    }
    async fn update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        todo!()
    }
    async fn cancel(
        &self,
        _request: Request<CancelRequest>,
    ) -> Result<Response<CancelResponse>, Status> {
        todo!()
    }
    async fn get(&self, _request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the query method.
    type queryStream = ReservationStream;
    /// for user to query reservations
    async fn query(
        &self,
        _request: Request<QueryRequest>,
    ) -> Result<Response<Self::queryStream>, Status> {
        todo!()
    }
    /// for admin to query reservations
    async fn filter(
        &self,
        _request: Request<FilterRequest>,
    ) -> Result<Response<FilterResponse>, Status> {
        todo!()
    }
    /// Server streaming response type for the listen method.
    type listenStream = ReservationStream;
    /// another system could monitor newly added/confirmed/cancelled reservations
    async fn listen(
        &self,
        _request: Request<ListenRequest>,
    ) -> Result<Response<Self::listenStream>, Status> {
        todo!()
    }
}
