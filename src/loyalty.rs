use chrono::{DateTime, NaiveDate, Utc};
use gzlib::id::LuhnCheck;
use packman::VecPackMember;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const TARGET_TO_JUMP: i32 = 50_000;

pub trait AccountExt
where
  Self: Sized,
{
  fn new(customer_id: u32, customer_birthdate: NaiveDate, created_by: u32) -> Self;
  fn set_card(&mut self, card_id: String) -> Result<&Self, String>;
  fn set_loyalty_level(&mut self, loyalty_level: LoyaltyLevel) -> &Self;
  fn set_birthdate(&mut self, birthdate: NaiveDate) -> &Self;
  fn burn_points(
    &mut self,
    purchase_id: Uuid,
    points_to_burn: i32,
    created_by: u32,
  ) -> Result<&Self, String>;
  fn close_purchase(
    &mut self,
    purchase_info: PurchaseInfo,
    created_by: u32,
  ) -> Result<PurchaseSummary, String>;
  fn get_balance(&self) -> i32;
  fn check_loyalty_level(&mut self);
  fn get_burned_points(&mut self, purchase_id: Uuid) -> i32;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
  pub account_id: Uuid, // Account ID cannot be changed later
  pub customer_id: u32, // customer ID
  pub customer_birthdate: NaiveDate,
  pub card_id: Option<String>,
  pub loyalty_level: LoyaltyLevel,
  pub balance_points: i32,
  pub yearly_gross_turnover: i32,
  pub transactions: Vec<Transaction>,
  pub created_by: u32,
  pub created_at: DateTime<Utc>,
}

impl AccountExt for Account {
  fn new(customer_id: u32, customer_birthdate: NaiveDate, created_by: u32) -> Self {
    Self {
      account_id: Uuid::new_v4(),
      customer_id,
      customer_birthdate,
      card_id: None,
      loyalty_level: LoyaltyLevel::L1,
      balance_points: 0,
      yearly_gross_turnover: 0,
      transactions: Vec::new(),
      created_by,
      created_at: Utc::now(),
    }
  }

  fn set_card(&mut self, card_id: String) -> Result<&Self, String> {
    // Check if Card ID is valid
    let card_id = card_id
      .luhn_check()
      .map_err(|_| "A megadott kártya azonosító nem valid!".to_string())?;
    // Set new card id
    self.card_id = Some(card_id);
    //Return Ok self ref
    Ok(self)
  }

  fn set_loyalty_level(&mut self, loyalty_level: LoyaltyLevel) -> &Self {
    self.loyalty_level = loyalty_level;
    self
  }

  fn set_birthdate(&mut self, birthdate: NaiveDate) -> &Self {
    self.customer_birthdate = birthdate;
    self
  }

  fn burn_points(
    &mut self,
    purchase_id: Uuid,
    points_to_burn: i32,
    created_by: u32,
  ) -> Result<&Self, String> {
    if self.get_balance() < points_to_burn {
      return Err(format!(
        "Nincs elég pont a tranzakcióhoz. Jelenlegi pont: {}",
        self.get_balance()
      ));
    }

    // Create new transaction object
    let transaction = Transaction::new(
      purchase_id,
      self.account_id.clone(),
      TransactionKind::Burn,
      points_to_burn,
      created_by,
    );

    // Update balance
    self.balance_points -= points_to_burn;

    // Push new transaction to transactions
    self.transactions.push(transaction);

    // Return Ok self ref
    Ok(self)
  }

  fn close_purchase(
    &mut self,
    purchase_info: PurchaseInfo,
    created_by: u32,
  ) -> Result<PurchaseSummary, String> {
    // Check if we should upgrade loyalty level
    self.check_loyalty_level();

    // Calculate points to earn
    let points_to_earn = (self.loyalty_level.get_discount_percentage()
      * purchase_info.payable_total_gross as f32)
      .round() as i32;

    // Create transaction
    let transaction = Transaction::new(
      purchase_info.purchase_id,
      self.account_id.clone(),
      TransactionKind::Earn {
        total_payable_amount: purchase_info.payable_total_gross as i32,
        discount: self.loyalty_level.get_discount_percentage(),
      },
      points_to_earn,
      created_by,
    );

    // Store transaction to transactions
    self.transactions.push(transaction);

    // Update balance
    self.balance_points += points_to_earn;

    // Update yearly turnover
    self.yearly_gross_turnover += purchase_info.payable_total_gross as i32;

    // Check if we should upgrade loyalty level
    self.check_loyalty_level();

    let burned_points = self.get_burned_points(purchase_info.purchase_id);
    let earned_points = points_to_earn;
    let balance_closing = self.get_balance();
    let balance_opening = balance_closing - earned_points + burned_points;

    Ok(PurchaseSummary {
      balance_opening,
      burned_points,
      earned_points,
      balance_closing,
    })
  }

  fn get_balance(&self) -> i32 {
    self.balance_points
  }

  fn check_loyalty_level(&mut self) {
    match self.loyalty_level {
      LoyaltyLevel::L1 => {
        // If balance is higher or eq with
        // the given target
        if self.get_balance() >= TARGET_TO_JUMP {
          self.loyalty_level = LoyaltyLevel::L2
        }
      }
      LoyaltyLevel::L2 => (), // Do nothing
    }
  }

  fn get_burned_points(&mut self, purchase_id: Uuid) -> i32 {
    self.transactions.iter().fold(0, |acc, t| {
      if t.purchase_id == purchase_id {
        match t.transaction_kind {
          TransactionKind::Burn => return acc + t.amount,
          _ => return acc,
        }
      }
      acc
    })
  }
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
      yearly_gross_turnover: 0,
      transactions: Vec::new(),
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

impl LoyaltyLevel {
  pub fn get_discount_percentage(&self) -> f32 {
    match self {
      LoyaltyLevel::L1 => 0.02,
      LoyaltyLevel::L2 => 0.04,
    }
  }
  pub fn from_str(str: &str) -> Result<Self, String> {
    match str {
      "l1" | "L1" => Ok(Self::L1),
      "l2" | "L2" => Ok(Self::L2),
      _ => Err("Nem megfelelő kedvezmény színt! L1, vagy L2".to_string()),
    }
  }
}

impl ToString for LoyaltyLevel {
  fn to_string(&self) -> String {
    match self {
      LoyaltyLevel::L1 => format!("L1"),
      LoyaltyLevel::L2 => format!("L2"),
    }
  }
}

pub struct PurchaseInfo {
  pub purchase_id: Uuid,
  pub payable_total_gross: u32,
  pub created_by: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TransactionKind {
  Earn {
    total_payable_amount: i32,
    discount: f32,
  },
  Burn,
}

impl Default for TransactionKind {
  fn default() -> Self {
    Self::Burn
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
  pub transaction_id: Uuid,
  pub account_id: Uuid,
  pub purchase_id: Uuid,
  pub transaction_kind: TransactionKind,
  pub amount: i32,
  pub crated_by: u32,
  pub created_at: DateTime<Utc>,
}

impl Transaction {
  pub fn new(
    purchase_id: Uuid,
    account_id: Uuid,
    transaction_kind: TransactionKind,
    amount: i32,
    crated_by: u32,
  ) -> Self {
    Self {
      transaction_id: Uuid::new_v4(),
      purchase_id,
      account_id,
      transaction_kind,
      amount,
      crated_by,
      created_at: Utc::now(),
    }
  }
}

impl Default for Transaction {
  fn default() -> Self {
    Self {
      transaction_id: Uuid::default(),
      purchase_id: Uuid::default(),
      account_id: Uuid::default(),
      transaction_kind: TransactionKind::default(),
      amount: 0,
      crated_by: 0,
      created_at: Utc::now(),
    }
  }
}

pub struct PurchaseSummary {
  pub balance_opening: i32,
  pub burned_points: i32,
  pub earned_points: i32,
  pub balance_closing: i32,
}
