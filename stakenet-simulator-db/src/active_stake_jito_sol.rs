use crate::{EpochBalanceResponse, big_decimal_u64::BigDecimalU64};
use sqlx::{Error, FromRow, Pool, Postgres, QueryBuilder, types::BigDecimal};

#[derive(FromRow)]
pub struct ActiveStakeJitoSol {
    #[sqlx(try_from = "BigDecimalU64")]
    pub epoch: u64,
    pub balance: BigDecimal,
}

impl ActiveStakeJitoSol {
    const NUM_FIELDS: u8 = 2;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO active_stake_jito_sol (epoch,balance) VALUES ";

    pub fn new(epoch: u64, balance: BigDecimal) -> Self {
        Self { epoch, balance }
    }

    pub async fn bulk_insert(
        db_connection: &Pool<Postgres>,
        records: Vec<Self>,
    ) -> Result<(), Error> {
        if records.is_empty() {
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
            separated.push_bind(BigDecimal::from(record.epoch));
            separated.push_bind(record.balance);
            separated.push_unseparated(") ");

            if num_records >= Self::INSERT_CHUNK_SIZE {
                query_builder.push(" ON CONFLICT (epoch) DO UPDATE SET balance = EXCLUDED.balance");
                let query = query_builder.build();
                query.execute(db_connection).await?;
                num_records = 0;
                query_builder = QueryBuilder::new(Self::INSERT_QUERY);
            }
        }

        if num_records > 0 {
            query_builder.push(" ON CONFLICT (epoch) DO UPDATE SET balance = EXCLUDED.balance");
            let query = query_builder.build();
            query.execute(db_connection).await?;
        }
        Ok(())
    }

    pub async fn fetch_balance_for_epoch_range(
        db_connection: &Pool<Postgres>,
        current_epoch: u64,
        lookback_period: u64,
    ) -> Result<EpochBalanceResponse, Error> {
        let start_epoch = current_epoch - lookback_period;
        let end_epoch = current_epoch;

        let query = "
            SELECT SUM(balance) as total_balance, COUNT(*) as record_count
            FROM active_stake_jito_sol 
            WHERE epoch >= $1 AND epoch <= $2
        ";

        let result: Option<(BigDecimal, i64)> = sqlx::query_as(query)
            .bind(BigDecimal::from(start_epoch))
            .bind(BigDecimal::from(end_epoch))
            .fetch_optional(db_connection)
            .await?;

        match result {
            Some((balance, count)) => Ok(EpochBalanceResponse { balance, count }),
            None => Err(Error::RowNotFound),
        }
    }

    pub async fn get_all_active_stakes(
        db_connection: &Pool<Postgres>,
    ) -> Result<Vec<ActiveStakeJitoSol>, Error> {
        let query = "SELECT * FROM active_stake_jito_sol";
        let result: Vec<ActiveStakeJitoSol> =
            sqlx::query_as(query).fetch_all(db_connection).await?;
        Ok(result)
    }

    pub async fn get_active_stakes_for_epoch_range(
        db_connection: &Pool<Postgres>,
        start_epoch: i64,
        end_epoch: i64,
    ) -> Result<Vec<ActiveStakeJitoSol>, Error> {
        let query = r#"
            SELECT *
            FROM active_stake_jito_sol
            WHERE epoch BETWEEN $1 AND $2;
        "#;

        let result: Vec<ActiveStakeJitoSol> = sqlx::query_as(query)
            .bind(start_epoch)
            .bind(end_epoch)
            .fetch_all(db_connection)
            .await?;

        Ok(result)
    }
}
