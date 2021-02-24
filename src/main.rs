use gzlib::proto::{
  self,
  loyalty::{
    loyalty_server::{Loyalty, LoyaltyServer},
    Purchase,
  },
};
pub use loyalty_microservice::{loyalty, loyalty::AccountExt, prelude};
use packman::VecPack;
use packman::*;
use prelude::*;
use std::error::Error;
use std::path::PathBuf;
use std::{env, str::FromStr};
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

struct LoyaltyService {
  accounts: Mutex<VecPack<loyalty::Account>>,
}

impl LoyaltyService {
  fn init(accounts: VecPack<loyalty::Account>) -> Self {
    Self {
      accounts: Mutex::new(accounts),
    }
  }
}

// Helper to try convert string to UUID
fn string_to_uuid(id: String) -> ServiceResult<Uuid> {
  Uuid::from_str(&id).map_err(|_| ServiceError::BadRequest(format!("A kért ID hibás: {}", id)))
}

#[tonic::async_trait]
impl Loyalty for LoyaltyService {
  async fn create_account(
    &self,
    request: Request<proto::loyalty::NewAccount>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn get_account_by_customer_id(
    &self,
    request: Request<proto::loyalty::CustomerRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn get_account_by_card_id(
    &self,
    request: Request<proto::loyalty::CardRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn get_account_by_query(
    &self,
    request: Request<proto::loyalty::QueryRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn get_purchase(
    &self,
    request: Request<proto::loyalty::GetPurchaseRequest>,
  ) -> Result<Response<proto::loyalty::Purchase>, Status> {
    todo!()
  }

  type GetPurchaseAllStream = tokio::sync::mpsc::Receiver<Result<Purchase, Status>>;

  async fn get_purchase_all(
    &self,
    request: Request<proto::loyalty::GetPurchaseAllRequest>,
  ) -> Result<Response<Self::GetPurchaseAllStream>, Status> {
    todo!()
  }

  async fn set_card(
    &self,
    request: Request<proto::loyalty::Card>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn set_loyalty_level(
    &self,
    request: Request<proto::loyalty::LoyaltyLevelRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn set_birthdate(
    &self,
    request: Request<proto::loyalty::SetBirthdateRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    todo!()
  }

  async fn add_purchase(
    &self,
    request: Request<proto::loyalty::PurchaseRequest>,
  ) -> Result<Response<proto::loyalty::Purchase>, Status> {
    todo!()
  }

  async fn remove_purchase(
    &self,
    request: Request<proto::loyalty::RemovePurchaseRequest>,
  ) -> Result<Response<proto::loyalty::Purchase>, Status> {
    todo!()
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // Init loyalty accounts database
  let loyalty_accounts: VecPack<loyalty::Account> =
    VecPack::load_or_init(PathBuf::from("data/loyalty_accounts"))
      .expect("Error while loading loyalty accounts db");

  let addr = env::var("SERVICE_ADDR_LOYALTY")
    .unwrap_or("[::1]:50075".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(LoyaltyServer::new(LoyaltyService::init(loyalty_accounts)))
      .serve_with_shutdown(addr, async {
        let _ = rx.await;
      })
      .await
      .unwrap()
  });

  tokio::signal::ctrl_c().await?;

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
