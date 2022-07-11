use crate::prelude::*;
use crate::transaction::{Kind, Transaction, TransactionId, UserId};
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::Serialize;
use std::cmp::max;
use std::collections::HashMap;

type UserTransactions = HashMap<TransactionId, Vec<TransactionTruncated>>;

/// A user's account state
#[derive(Default, Debug)]
pub struct UserState {
    /// The total funds that are held for dispute. This should be equal to total - available amounts
    held: Decimal,

    /// The total funds that are available or held. This should be equal to available + held
    total: Decimal,

    /// Whether the account is locked. An account is locked if a charge back occurs
    locked: Lock,

    transactions: HashMap<TransactionId, Vec<TransactionTruncated>>,
}

impl UserState {
    /// The total funds that are available for trading, staking, withdrawal, etc. This should be equal to the total - held amounts
    fn available(&self) -> Decimal {
        max(self.total - self.held, Decimal::ZERO)
    }
}

#[derive(Serialize)]
pub struct UserSummary {
    client: UserId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: Lock,
}

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub enum Lock {
    #[default]
    #[serde(rename(serialize = "false"))]
    Unlocked,

    #[serde(rename(serialize = "true"))]
    Locked,
}

#[derive(Default, Debug)]
pub struct TransactionEngine {
    user_states: DashMap<UserId, UserState>,
}

impl TransactionEngine {
    pub fn add_transaction(&self, new: Transaction) -> Result<()> {
        if !self.user_states.contains_key(&new.client) {
            self.user_states.insert(new.client, UserState::default());
        }

        let mut user_state = self.user_states.get_mut(&new.client).unwrap();
        match new.kind {
            Kind::Deposit => {
                user_state.total += new.amount.ok_or_else(|| {
                    anyhow!(
                        "There must be an amount for transactions of type deposit: {:?}",
                        new
                    )
                })?;
            }
            Kind::Withdrawal => {
                let new_amount = new.amount.ok_or_else(|| {
                    anyhow!(
                        "There must be an amount for transactions of type withrawal: {:?}",
                        new
                    )
                })?;

                if user_state.available() < new_amount {
                    warn!("Withdraw failed! User {} has only {} funds available, and cannot withdraw {} (for transaction {}).", new.client, user_state.available(), new_amount, new.transaction_id);
                } else {
                    user_state.total -= new_amount;
                }
            }
            Kind::Dispute => {
                let transaction = get_first_transaction_by_kind(
                    &user_state.transactions,
                    new.transaction_id,
                    Kind::Deposit,
                );

                let transaction = match transaction {
                    Some(t) => t,
                    None => {
                        warn!("Cannot dispute a transaction that does not exist, ignoring: transaction {} for user {}", new.transaction_id,new.client);
                        return Ok(());
                    }
                };

                if transaction.kind != Kind::Deposit {
                    warn!("First transaction (for a transaction ID) must be a deposit");
                    return Ok(());
                }

                let amount = transaction
                    .amount
                    .ok_or_else(|| anyhow!("Deposit should have an amount!"))?;

                if user_state.total < user_state.held + amount {
                    warn!(
                        "Amount held is now greater than total! total={}, held={}, amount={}",
                        user_state.total, user_state.held, amount
                    );
                }

                user_state.held += amount;
            }
            Kind::Resolve => {
                if get_first_transaction_by_kind(
                    &user_state.transactions,
                    new.transaction_id,
                    Kind::Dispute,
                )
                .is_none()
                {
                    warn!("Cannot resolve a dispute that does not exist, ignoring: transaction {} for user {}", new.transaction_id, new.client);
                    return Ok(());
                }

                let deposit = get_first_transaction_by_kind(
                    &user_state.transactions,
                    new.transaction_id,
                    Kind::Deposit,
                );

                let deposit = match deposit {
                    Some(t) => t,
                    None => {
                        warn!("Cannot resolve a dispute that has no corresponding deposit, ignoring: transaction {} for user {}", new.transaction_id, new.client);
                        return Ok(());
                    }
                };

                let amount = deposit
                    .amount
                    .ok_or_else(|| anyhow!("Deposit should have an amount!"))?;

                if user_state.held - amount < Decimal::ZERO {
                    return Err(anyhow!(
                        "amount is more than held! amount={} held={}",
                        user_state.held,
                        amount
                    ));
                }

                user_state.held -= amount;
            }
            Kind::Chargeback => {
                if get_first_transaction_by_kind(
                    &user_state.transactions,
                    new.transaction_id,
                    Kind::Dispute,
                )
                .is_none()
                {
                    warn!("Cannot chargeback a dispute that does not exist, ignoring: transaction {} for user {}", new.transaction_id, new.client);
                    return Ok(());
                }

                let deposit = get_first_transaction_by_kind(
                    &user_state.transactions,
                    new.transaction_id,
                    Kind::Deposit,
                );

                let deposit = match deposit {
                    Some(t) => t,
                    None => {
                        warn!("Cannot chargeback a dispute that has no corresponding deposit, ignoring: transaction {} for user {}", new.transaction_id, new.client);
                        return Ok(());
                    }
                };

                let amount = deposit
                    .amount
                    .ok_or_else(|| anyhow!("Deposit should have an amount!"))?;

                if user_state.held - amount < Decimal::ZERO {
                    return Err(anyhow!(
                        "amount is more than held! amount={} held={}",
                        user_state.held,
                        amount
                    ));
                }

                if user_state.total - amount < Decimal::ZERO {
                    warn!("user account {} went negative", new.client);
                }

                user_state.held -= amount;
                user_state.total -= amount;
                user_state.locked = Lock::Locked;
            }
        }

        // If previous match ran (and didn't return out), then we can add the transaction to
        // the transaction list for the user
        let user_transactions = user_state
            .transactions
            .entry(new.transaction_id)
            .or_default();

        user_transactions.push(TransactionTruncated {
            kind: new.kind,
            amount: new.amount,
        });

        Ok(())
    }
    pub fn current_account_states(&self) -> Vec<UserSummary> {
        self.user_states
            .iter()
            .map(|state| UserSummary {
                client: *state.key(),
                available: state.available(),
                held: state.held,
                total: state.total,
                locked: state.locked,
            })
            .collect()
    }
}

fn get_first_transaction_by_kind(
    user_transactions: &UserTransactions,
    transaction_id: TransactionId,
    kind: Kind,
) -> Option<&TransactionTruncated> {
    user_transactions
        .get(&transaction_id)
        .and_then(|v| v.iter().find(|t| t.kind == kind))
}

#[derive(Debug)]
struct TransactionTruncated {
    pub kind: Kind,
    pub amount: Option<Decimal>,
}

#[test]
fn input_from_pdf() {
    let _input = r"
    type,         client,   tx,   amount
    deposit,           1,    1,      1.0
    deposit,           2,    2,      2.0
    deposit,           1,    3,      2.0
    withdrawal,        1,    4,      1.5
    withdrawal,        2,    5,      3.0
";
}

#[test]
fn parsed_input_from_pdf() -> Result<()> {
    let engine = TransactionEngine::default();

    engine.add_transaction(Transaction {
        kind: Kind::Deposit,
        client: 1,
        transaction_id: 1,
        amount: Some(Decimal::from(1)),
    })?;

    engine.add_transaction(Transaction {
        kind: Kind::Deposit,
        client: 2,
        transaction_id: 2,
        amount: Some(Decimal::from(2)),
    })?;

    engine.add_transaction(Transaction {
        kind: Kind::Deposit,
        client: 1,
        transaction_id: 3,
        amount: Some(Decimal::from(2)),
    })?;

    engine.add_transaction(Transaction {
        kind: Kind::Withdrawal,
        client: 1,
        transaction_id: 4,
        amount: Some(Decimal::from_str_exact("1.5").unwrap()),
    })?;

    engine.add_transaction(Transaction {
        kind: Kind::Withdrawal,
        client: 2,
        transaction_id: 5,
        amount: Some(Decimal::from(3)),
    })?;

    Ok(())
}

#[test]
fn simple_dispute() -> Result<()> {
    let engine = TransactionEngine::default();
    engine.add_transaction(Transaction {
        kind: Kind::Deposit,
        client: 0,
        transaction_id: 0,
        amount: Some(Decimal::from(2)),
    })?;

    engine.add_transaction(Transaction {
        kind: Kind::Dispute,
        client: 0,
        transaction_id: 0,
        amount: None,
    })?;

    Ok(())
}
