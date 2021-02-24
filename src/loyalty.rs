use chrono::{DateTime, NaiveDate, Utc};
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait AccountExt {}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
  account_id: Uuid, // Account ID cannot be changed later
  customer_id: u32, // customer ID
  customer_birthdate: NaiveDate,
  card_id: Option<String>,
  loyalty_level: LoyaltyLevel,
  balance_points: i32,
  purchases: Vec<Purchase>,
  created_by: u32,
  created_at: DateTime<Utc>,
}

impl Default for Account {
  fn default() -> Self {
    Self {
      account_id: Uuid::default(),
      customer_id: 0,
      customer_birthdate: Utc::today().naive_utc(),
      card_id: None,
      loyalty_level: LoyaltyLevel::default(),
      balance_points: 0,
      purchases: Vec::new(),
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl VecPackMember for Account {
  type Out = Uuid;

  fn get_id(&self) -> &Self::Out {
    &self.account_id
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LoyaltyLevel {
  L1, // 2%
  L2, // 4%
}

impl Default for LoyaltyLevel {
  fn default() -> Self {
    Self::L1
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Purchase {
  purchase_id: Uuid,
  payable_total_gross: i32,
  balance_opening: i32,
  burned_points: i32,
  earned_points: i32,
  balance_closing: i32,
  crated_by: u32,
  created_at: DateTime<Utc>,
}

impl Default for Purchase {
  fn default() -> Self {
    Self {
      purchase_id: Uuid::default(),
      payable_total_gross: 0,
      balance_opening: 0,
      burned_points: 0,
      earned_points: 0,
      balance_closing: 0,
      crated_by: 0,
      created_at: Utc::now(),
    }
  }
}
