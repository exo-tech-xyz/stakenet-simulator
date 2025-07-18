use crate::big_decimal_u64::BigDecimalU64;
use sqlx::{Error, FromRow, Pool, Postgres, QueryBuilder, types::BigDecimal};

#[derive(Debug, FromRow)]
pub struct EpochPriorityFees {
    pub id: String,
    pub identity_pubkey: String,
    #[sqlx(try_from = "BigDecimalU64")]
    pub epoch: u64,
    #[sqlx(try_from = "BigDecimalU64")]
    pub priority_fees: u64,
}

impl EpochPriorityFees {
    const NUM_FIELDS: u8 = 4;
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str =
        "INSERT INTO epoch_priority_fees (id, identity_pubkey, epoch, priority_fees) VALUES ";

    pub fn new(identity: String, epoch: u64, priority_fees: u64) -> Self {
        Self {
            id: format!("{}-{}", epoch, identity),
            identity_pubkey: identity,
            epoch: epoch,
            priority_fees: priority_fees,
        }
    }

    pub async fn bulk_insert(
        db_connection: &Pool<Postgres>,
        records: Vec<Self>,
    ) -> Result<u64, Error> {
        if records.len() <= 0 {
            return Ok(0);
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(Self::INSERT_QUERY);
        let mut num_records: usize = 0;
        let mut rows_affected = 0u64;

        // Bind all values
        for record in records.into_iter() {
            num_records += 1;
            if num_records > 1 {
                query_builder.push(", (");
            } else {
                query_builder.push("(");
            }
            let mut separated = query_builder.separated(", ");
            separated.push_bind(record.id);
            separated.push_bind(record.identity_pubkey);
            separated.push_bind(BigDecimal::from(record.epoch));
            separated.push_bind(BigDecimal::from(record.priority_fees));

            separated.push_unseparated(") ");

            if num_records >= Self::INSERT_CHUNK_SIZE {
                query_builder.push(" ON CONFLICT (id) DO NOTHING");
                let query = query_builder.build();
                let res = query.execute(db_connection).await?;
                rows_affected += res.rows_affected();
                num_records = 0;
                query_builder = QueryBuilder::new(Self::INSERT_QUERY);
            }
        }

        if num_records > 0 {
            query_builder.push(" ON CONFLICT (id) DO NOTHING");
            let query = query_builder.build();
            let res = query.execute(db_connection).await?;
            rows_affected += res.rows_affected();
        }

        Ok(rows_affected)
    }
}
