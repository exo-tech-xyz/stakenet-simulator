use sqlx::{Error as SqlxError, Pool, Postgres, QueryBuilder, types::BigDecimal};
use validator_history::ValidatorHistory as JitoValidatorHistory;

pub struct ValidatorHistory {
    pub struct_version: u32,
    pub vote_account: String,
    pub index: u32,
    pub bump: u8,
    pub last_ip_timestamp: u64,
    pub last_version_timestamp: u64,
}

impl From<JitoValidatorHistory> for ValidatorHistory {
    fn from(value: JitoValidatorHistory) -> Self {
        Self {
            struct_version: value.struct_version,
            vote_account: value.vote_account.to_string(),
            index: value.index,
            bump: value.bump,
            last_ip_timestamp: value.last_ip_timestamp,
            last_version_timestamp: value.last_version_timestamp,
        }
    }
}

impl ValidatorHistory {
    const NUM_FIELDS: u8 = 6;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO validator_histories (vote_account,struct_version,index,bump,last_ip_timestamp,last_version_timestamp) VALUES ";

    pub async fn bulk_insert(
        db_connection: &Pool<Postgres>,
        records: Vec<Self>,
    ) -> Result<(), SqlxError> {
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
            separated.push_bind(record.vote_account);
            separated.push_bind(BigDecimal::from(record.struct_version));
            separated.push_bind(BigDecimal::from(record.index));
            separated.push_bind(i16::from(record.bump));
            separated.push_bind(BigDecimal::from(record.last_ip_timestamp));
            separated.push_bind(BigDecimal::from(record.last_version_timestamp));

            separated.push_unseparated(") ");

            if num_records >= Self::INSERT_CHUNK_SIZE {
                query_builder.push(" ON CONFLICT (vote_account) DO NOTHING");
                let query = query_builder.build();
                query.execute(db_connection).await?;
                num_records = 0;
                query_builder = QueryBuilder::new(Self::INSERT_QUERY);
            }
        }

        if num_records > 0 {
            query_builder.push(" ON CONFLICT (vote_account) DO NOTHING");
            let query = query_builder.build();
            query.execute(db_connection).await?;
        }
        Ok(())
    }
}
