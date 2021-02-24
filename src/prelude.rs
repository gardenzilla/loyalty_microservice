use gzlib::proto::loyalty::{transaction::TransactionKind, Account, Transaction};

pub enum ServiceError {
  InternalError(String),
  NotFound(String),
  AlreadyExists(String),
  BadRequest(String),
}

impl ServiceError {
  pub fn internal_error(msg: &str) -> Self {
    ServiceError::InternalError(msg.to_string())
  }
  pub fn not_found(msg: &str) -> Self {
    ServiceError::NotFound(msg.to_string())
  }
  pub fn already_exist(msg: &str) -> Self {
    ServiceError::AlreadyExists(msg.to_string())
  }
  pub fn bad_request(msg: &str) -> Self {
    ServiceError::BadRequest(msg.to_string())
  }
}

impl std::fmt::Display for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ServiceError::InternalError(msg) => write!(f, "{}", msg),
      ServiceError::NotFound(msg) => write!(f, "{}", msg),
      ServiceError::AlreadyExists(msg) => write!(f, "{}", msg),
      ServiceError::BadRequest(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::fmt::Debug for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("")
      .field(&"ServiceError".to_string())
      .field(self)
      .finish()
  }
}

impl From<ServiceError> for ::tonic::Status {
  fn from(error: ServiceError) -> Self {
    match error {
      ServiceError::InternalError(msg) => ::tonic::Status::internal(msg),
      ServiceError::NotFound(msg) => ::tonic::Status::not_found(msg),
      ServiceError::AlreadyExists(msg) => ::tonic::Status::already_exists(msg),
      ServiceError::BadRequest(msg) => ::tonic::Status::invalid_argument(msg),
    }
  }
}

impl From<::packman::PackError> for ServiceError {
  fn from(error: ::packman::PackError) -> Self {
    match error {
      ::packman::PackError::ObjectNotFound => ServiceError::not_found(&error.to_string()),
      _ => ServiceError::internal_error(&error.to_string()),
    }
  }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl From<std::env::VarError> for ServiceError {
  fn from(error: std::env::VarError) -> Self {
    ServiceError::internal_error(&format!("ENV KEY NOT FOUND. {}", error))
  }
}

impl From<crate::loyalty::Account> for Account {
  fn from(f: crate::loyalty::Account) -> Self {
    Self {
      account_id: f.account_id.to_string(),
      customer_id: f.customer_id,
      customer_birthdate: f.customer_birthdate.to_string(),
      card_id: match f.card_id {
        Some(card_id) => card_id,
        None => "".to_string(),
      },
      loyalty_level: f.loyalty_level.to_string(),
      balance_points: f.balance_points,
      yearly_gross_turnover: f.yearly_gross_turnover,
      created_at: f.created_at.to_rfc3339(),
      created_by: f.created_by,
    }
  }
}

impl From<crate::loyalty::Transaction> for Transaction {
  fn from(f: crate::loyalty::Transaction) -> Self {
    Self {
      transaction_id: f.transaction_id.to_string(),
      account_id: f.account_id.to_string(),
      purchase_id: f.purchase_id.to_string(),
      transaction_kind: match f.transaction_kind {
        crate::loyalty::TransactionKind::Earn {
          total_payable_amount: _,
          discount: _,
        } => TransactionKind::Earn,
        crate::loyalty::TransactionKind::Burn => TransactionKind::Burn,
      } as i32,
      amount: f.amount,
      created_by: f.crated_by,
      created_at: f.created_at.to_rfc3339(),
    }
  }
}
