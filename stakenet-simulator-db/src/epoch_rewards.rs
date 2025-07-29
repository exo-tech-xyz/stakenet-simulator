use crate::big_decimal_u64::BigDecimalU64;
use num_traits::ToPrimitive;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use sqlx::{Error, FromRow, Pool, Postgres, QueryBuilder, types::BigDecimal};

const MAX_BPS: u64 = 10_000;

#[derive(FromRow)]
pub struct EpochRewards {
    pub id: String,
    pub vote_pubkey: String,
    #[sqlx(try_from = "BigDecimalU64")]
    pub epoch: u64,
    #[sqlx(try_from = "i16")]
    pub inflation_commission_bps: u16,
    #[sqlx(try_from = "BigDecimalU64")]
    pub total_inflation_rewards: u64,
    #[sqlx(try_from = "i16")]
    pub mev_commission_bps: u16,
    #[sqlx(try_from = "BigDecimalU64")]
    pub total_mev_rewards: u64,
    #[sqlx(try_from = "i16")]
    pub priority_fee_commission_bps: u16,
    #[sqlx(try_from = "BigDecimalU64")]
    pub total_priority_fee_rewards: u64,
    #[sqlx(try_from = "BigDecimalU64")]
    pub active_stake: u64,
}

impl EpochRewards {
    const NUM_FIELDS: u8 = 10;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO epoch_rewards (id,vote_pubkey,epoch,inflation_commission_bps,total_inflation_rewards,mev_commission_bps,total_mev_rewards,priority_fee_commission_bps,total_priority_fee_rewards,active_stake) VALUES ";

    pub async fn bulk_insert(
        db_connection: &Pool<Postgres>,
        records: Vec<Self>,
    ) -> Result<(), Error> {
        if records.len() <= 0 {
            return Ok(());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(Self::INSERT_QUERY);
        let mut num_records: usize = 0;

        for record in records.into_iter() {
            num_records += 1;
            if num_records > 1 {
                query_builder.push(", (");
            } else {
                query_builder.push("(");
            }
            let mut separated = query_builder.separated(", ");
            separated.push_bind(record.id);
            separated.push_bind(record.vote_pubkey);
            separated.push_bind(BigDecimal::from(record.epoch));
            separated.push_bind(i32::from(record.inflation_commission_bps));
            separated.push_bind(BigDecimal::from(record.total_inflation_rewards));
            separated.push_bind(i32::from(record.mev_commission_bps));
            separated.push_bind(BigDecimal::from(record.total_mev_rewards));
            separated.push_bind(i32::from(record.priority_fee_commission_bps));
            separated.push_bind(BigDecimal::from(record.total_priority_fee_rewards));
            separated.push_bind(BigDecimal::from(record.active_stake));
            separated.push_unseparated(") ");

            if num_records >= Self::INSERT_CHUNK_SIZE {
                query_builder.push(" ON CONFLICT (id) DO NOTHING");
                let query = query_builder.build();
                query.execute(db_connection).await?;
                num_records = 0;
                query_builder = QueryBuilder::new(Self::INSERT_QUERY);
            }
        }

        if num_records > 0 {
            let query = query_builder.build();
            query.execute(db_connection).await?;
        }
        Ok(())
    }

    pub async fn fetch_for_validators_and_epochs(
        db_connection: &Pool<Postgres>,
        vote_accounts: &Vec<String>,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<Vec<Self>, Error> {
        sqlx::query_as::<_, Self>(&format!(
            "SELECT * FROM epoch_rewards WHERE vote_pubkey = ANY($1) AND epoch BETWEEN $2 AND $3",
        ))
        .bind(vote_accounts)
        .bind(BigDecimal::from(start_epoch))
        .bind(BigDecimal::from(end_epoch))
        .fetch_all(db_connection)
        .await
    }

    /// Returns the APY as a fp 
    // TODO: Currently it's a simple APR (not accounting for compounding epoch over epoch)
    pub fn apy(&self) -> Option<f64> {
        let inflation_for_stakers = self.total_inflation_rewards * (MAX_BPS - u64::from(self.inflation_commission_bps)) / MAX_BPS;
        let inflation_for_epoch = (inflation_for_stakers.to_f64()? / LAMPORTS_PER_SOL.to_f64()?) / (self.active_stake.to_f64()? / LAMPORTS_PER_SOL.to_f64()?);
        // REVIEW: Is there a better way to annualize? Maybe include compounding
        // Annualize assuming epochs are 2 days 
        let inflation_apy = inflation_for_epoch * (365.0 / 2.0);

        let mev_for_stakers = self.total_mev_rewards * (MAX_BPS - u64::from(self.mev_commission_bps)) / MAX_BPS;
        let mev_for_epoch = (mev_for_stakers.to_f64()? / LAMPORTS_PER_SOL.to_f64()?) / (self.active_stake.to_f64()? / LAMPORTS_PER_SOL.to_f64()?);
        let mev_apy = mev_for_epoch * (365.0 / 2.0);

        let priority_fee_for_stakers = self.total_priority_fee_rewards * (MAX_BPS - u64::from(self.priority_fee_commission_bps)) / MAX_BPS;
        let priority_fee_for_epoch = (priority_fee_for_stakers.to_f64()? / LAMPORTS_PER_SOL.to_f64()?) / (self.active_stake.to_f64()? / LAMPORTS_PER_SOL.to_f64()?);
        let priority_fee_apy = priority_fee_for_epoch * (365.0 / 2.0);
        
        Some(inflation_apy + mev_apy + priority_fee_apy)
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::pubkey::Pubkey;

    use super::*;

    #[test]
    fn test_apy() {
        let rewards = EpochRewards {
            id: "".to_string(),
            vote_pubkey: Pubkey::new_unique().to_string(),
            epoch: 1,
            inflation_commission_bps: 500,
            total_inflation_rewards: 1_000_000,
            mev_commission_bps: 1_000,
            total_mev_rewards: 1_000_000,
            priority_fee_commission_bps: 10_000,
            total_priority_fee_rewards: 1_000_000,
            active_stake: 1_000_000_000,
        };

        let actual = rewards.apy();
        assert_eq!(actual, Some(0.337625))
    }
}
