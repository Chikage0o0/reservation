use abi::{
    reservation_service_server::ReservationService, CancelRequest, CancelResponse, ConfirmRequest,
    ConfirmResponse, FilterRequest, FilterResponse, GetRequest, GetResponse, ListenRequest,
    QueryRequest, ReserveRequest, ReserveResponse, UpdateRequest, UpdateResponse,
};
use anyhow::Result;
use reservation::{ReservationManager, Rsvp as _};

use tonic::{Request, Response, Status};

use crate::ReservationStream;

pub struct RsvpService {
    manager: ReservationManager,
}

impl RsvpService {
    pub async fn from_config(config: &abi::config::Config) -> Result<Self> {
        let manager = ReservationManager::from_config(&config.db).await?;
        Ok(Self { manager })
    }

    pub fn new(manager: ReservationManager) -> Self {
        Self { manager }
    }
}

#[tonic::async_trait]
impl ReservationService for RsvpService {
    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<ReserveResponse>, Status> {
        let request: ReserveRequest = request.into_inner();
        match request.reservation {
            Some(rsvp) => {
                let rsvp = self.manager.reserve(rsvp).await?;
                Ok(Response::new(ReserveResponse {
                    reservation: Some(rsvp),
                }))
            }
            None => Err(Status::invalid_argument("Invalid reservation")),
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    async fn test_reserve(pool: sqlx::PgPool) {
        let manager = ReservationManager::new(pool);
        let service = RsvpService::new(manager);
        let request = ReserveRequest {
            reservation: Some(abi::Reservation::new_pendding(
                "user".to_string(),
                "room".to_string(),
                "2021-01-01T00:00:00Z".parse().unwrap(),
                "2021-01-02T00:00:00Z".parse().unwrap(),
                "note",
            )),
        };
        let response = service.reserve(Request::new(request)).await.unwrap();
        assert_eq!(response.get_ref().reservation.as_ref().unwrap().id, 1);
    }
}
