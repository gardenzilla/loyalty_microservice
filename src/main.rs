use chrono::NaiveDate;
use gzlib::proto::{
  self,
  loyalty::{
    loyalty_server::{Loyalty, LoyaltyServer},
    Account, BurnRequest, Card, CardRequest, ClosePurchaseRequest, CustomerRequest,
    LoyaltyLevelRequest, NewAccount, PurchaseSummary, QueryRequest, SetBirthdateRequest,
    Transaction, TransactionAllRequest,
  },
};
pub use loyalty_microservice::{loyalty, loyalty::AccountExt, prelude};
use packman::VecPack;
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

  async fn create_account(&self, r: NewAccount) -> ServiceResult<Account> {
    // Check if customer already has an account
    if self
      .accounts
      .lock()
      .await
      .iter()
      .find(|a| a.unpack().customer_id == r.customer_id)
      .is_some()
    {
      return Err(ServiceError::bad_request(
        "A megadott vásárlónak már van törzsvásárlói fiókja!",
      ));
    }

    // Convert String to NaiveDate
    let birthdate = NaiveDate::parse_from_str(&r.birthdate, "%Y-%m-%d")
      .map_err(|_| ServiceError::bad_request("A megadott születési dátum hibás formátumú!"))?;

    // Create new account
    let new_account = loyalty::Account::new(r.customer_id, birthdate, r.created_by);

    // Add new account to DB
    self.accounts.lock().await.insert(new_account.clone())?;

    // Get by ID
    let res = self
      .accounts
      .lock()
      .await
      .find_id(&new_account.account_id)?
      .unpack()
      .clone();

    Ok(res.into())
  }

  async fn get_account_by_customer_id(&self, r: CustomerRequest) -> ServiceResult<Account> {
    let res = self
      .accounts
      .lock()
      .await
      .iter()
      .find(|a| a.unpack().customer_id == r.customer_id)
      .ok_or(ServiceError::bad_request(
        "A megadott vásárlónak nincs törzsvásárlói fiókja",
      ))?
      .unpack()
      .clone();

    Ok(res.into())
  }

  async fn get_account_by_card_id(&self, r: CardRequest) -> ServiceResult<Account> {
    let res = self
      .accounts
      .lock()
      .await
      .iter()
      .find(|a| match &a.unpack().card_id {
        Some(_card_id) => _card_id == &r.card_id,
        None => false,
      })
      .ok_or(ServiceError::bad_request(
        "A megadott kártyához nem tartozik törzsvásárlói fiók",
      ))?
      .unpack()
      .clone();
    Ok(res.into())
  }

  async fn get_account_by_query(&self, r: QueryRequest) -> ServiceResult<Account> {
    // Convert birthdate to NaiveDate
    let birthdate = NaiveDate::parse_from_str(&r.birthdate, "%Y-%m-%d").map_err(|_| {
      ServiceError::bad_request("A megadott születési dátum nem megfelelő formátumú")
    })?;

    // Try to find account
    let res = self
      .accounts
      .lock()
      .await
      .iter()
      .find(|a| {
        (a.unpack().customer_id == r.customer_id) && (a.unpack().customer_birthdate == birthdate)
      })
      .ok_or(ServiceError::not_found(
        "A kért fiók nem található a megadott adatok alapján!",
      ))?
      .unpack()
      .clone();

    Ok(res.into())
  }

  async fn get_transactions_all(
    &self,
    r: TransactionAllRequest,
  ) -> ServiceResult<Vec<Transaction>> {
    let res = self
      .accounts
      .lock()
      .await
      .find_id(&string_to_uuid(r.account_id)?)?
      .unpack()
      .transactions
      .iter()
      .map(|t| t.clone().into())
      .collect::<Vec<Transaction>>();
    Ok(res)
  }

  async fn set_card(&self, r: Card) -> ServiceResult<Account> {
    let res = self
      .accounts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.set_to_account_id)?)?
      .as_mut()
      .unpack()
      .set_card(r.card_id)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn set_loyalty_level(&self, r: LoyaltyLevelRequest) -> ServiceResult<Account> {
    let res = self
      .accounts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.account_id)?)?
      .as_mut()
      .unpack()
      .set_loyalty_level(
        loyalty::LoyaltyLevel::from_str(&r.loyalty_level)
          .map_err(|e| ServiceError::bad_request(&e))?,
      )
      .clone();
    Ok(res.into())
  }

  async fn set_birthdate(&self, r: SetBirthdateRequest) -> ServiceResult<Account> {
    // Convert birthdate to NaiveDate
    let birthdate = NaiveDate::parse_from_str(&r.birthdate, "%Y-%m-%d").map_err(|_| {
      ServiceError::bad_request("A megadott születési dátum nem megfelelő formátumú")
    })?;

    let res = self
      .accounts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.account_id)?)?
      .as_mut()
      .unpack()
      .set_birthdate(birthdate)
      .clone();
    Ok(res.into())
  }

  async fn burn_points(&self, r: BurnRequest) -> ServiceResult<Transaction> {
    let res = self
      .accounts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.account_id)?)?
      .as_mut()
      .unpack()
      .burn_points(
        string_to_uuid(r.purchase_id)?,
        r.points_to_burn,
        r.created_by,
      )
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    Ok(res.into())
  }

  async fn close_purchase(&self, r: ClosePurchaseRequest) -> ServiceResult<PurchaseSummary> {
    let summary = self
      .accounts
      .lock()
      .await
      .find_id_mut(&string_to_uuid(r.account_id.clone())?)?
      .as_mut()
      .unpack()
      .close_purchase(
        loyalty::PurchaseInfo {
          purchase_id: string_to_uuid(r.purchase_id.clone())?,
          payable_total_gross: r.total_gross,
          created_by: r.created_by,
        },
        r.created_by,
      )
      .map_err(|e| ServiceError::bad_request(&e))?;

    Ok(PurchaseSummary {
      account_id: r.account_id,
      purchase_id: r.purchase_id,
      balance_opening: summary.balance_opening,
      burned_points: summary.burned_points,
      earned_points: summary.earned_points,
      balance_closing: summary.balance_closing,
    })
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
    let res = self.create_account(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn get_account_by_customer_id(
    &self,
    request: Request<proto::loyalty::CustomerRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self
      .get_account_by_customer_id(request.into_inner())
      .await?;
    Ok(Response::new(res))
  }

  async fn get_account_by_card_id(
    &self,
    request: Request<proto::loyalty::CardRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self.get_account_by_card_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn get_account_by_query(
    &self,
    request: Request<proto::loyalty::QueryRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self.get_account_by_query(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetTransactionsAllStream = tokio::sync::mpsc::Receiver<Result<Transaction, Status>>;

  async fn get_transactions_all(
    &self,
    request: Request<proto::loyalty::TransactionAllRequest>,
  ) -> Result<Response<Self::GetTransactionsAllStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_transactions_all(request.into_inner()).await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn set_card(
    &self,
    request: Request<proto::loyalty::Card>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self.set_card(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn set_loyalty_level(
    &self,
    request: Request<proto::loyalty::LoyaltyLevelRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self.set_loyalty_level(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn set_birthdate(
    &self,
    request: Request<proto::loyalty::SetBirthdateRequest>,
  ) -> Result<Response<proto::loyalty::Account>, Status> {
    let res = self.set_birthdate(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn burn_points(
    &self,
    request: Request<proto::loyalty::BurnRequest>,
  ) -> Result<Response<proto::loyalty::Transaction>, Status> {
    let res = self.burn_points(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn close_purchase(
    &self,
    request: Request<proto::loyalty::ClosePurchaseRequest>,
  ) -> Result<Response<proto::loyalty::PurchaseSummary>, Status> {
    let res = self.close_purchase(request.into_inner()).await?;
    Ok(Response::new(res))
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
