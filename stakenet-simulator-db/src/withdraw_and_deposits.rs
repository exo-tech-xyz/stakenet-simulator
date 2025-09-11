use crate::big_decimal_u64::BigDecimalU64;
use sqlx::{Error, FromRow, Pool, Postgres, QueryBuilder, types::BigDecimal};

#[derive(FromRow, Debug)]
pub struct WithdrawsAndDeposits {
    pub id: String, // {epoch}-{vote_pubkey}
    #[sqlx(try_from = "BigDecimalU64")]
    pub epoch: u64,
    pub vote_pubkey: String,
    pub withdraw_stake: BigDecimal,
    pub deposit_stake: BigDecimal,
}

impl WithdrawsAndDeposits {
    const NUM_FIELDS: u8 = 5;
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO withdraws_and_deposits \
        (id, epoch, vote_pubkey, withdraw_stake, deposit_stake) VALUES ";

    pub fn new(
        epoch: u64,
        vote_pubkey: String,
        withdraw_stake: BigDecimal,
        deposit_stake: BigDecimal,
    ) -> Self {
        let id = format!("{}-{}", epoch, vote_pubkey);
        Self {
            id,
            epoch,
            vote_pubkey,
            withdraw_stake,
            deposit_stake,
        }
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
            separated.push_bind(record.id);
            separated.push_bind(BigDecimal::from(record.epoch));
            separated.push_bind(record.vote_pubkey);
            separated.push_bind(record.withdraw_stake);
            separated.push_bind(record.deposit_stake);
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
            query_builder.push(" ON CONFLICT (id) DO NOTHING");
            let query = query_builder.build();
            query.execute(db_connection).await?;
        }
        Ok(())
    }

    pub async fn get_details_for_epoch(
        db_connection: &Pool<Postgres>,
        epoch: u64,
    ) -> Result<Vec<Self>, Error> {
        let query = "
            SELECT id, epoch, vote_pubkey, withdraw_stake, deposit_stake
            FROM withdraws_and_deposits
            WHERE epoch = $1
        ";

        sqlx::query_as::<_, Self>(query)
            .bind(BigDecimal::from(epoch))
            .fetch_all(db_connection)
            .await
    }
}
