use crate::big_decimal_u64::BigDecimalU64;

use sqlx::{Error as SqlxError, Pool, Postgres, QueryBuilder, prelude::FromRow, types::BigDecimal};
use validator_history::ClusterHistoryEntry as JitoClusterHistoryEntry;

#[derive(FromRow)]
pub struct ClusterHistoryEntry {
    #[sqlx(try_from = "i32")]
    pub epoch: u16,
    #[sqlx(try_from = "i64")]
    pub total_blocks: u32,
    #[sqlx(try_from = "BigDecimalU64")]
    pub epoch_start_timestamp: u64,
}

impl From<JitoClusterHistoryEntry> for ClusterHistoryEntry {
    fn from(value: JitoClusterHistoryEntry) -> Self {
        Self {
            epoch: value.epoch,
            total_blocks: value.total_blocks,
            epoch_start_timestamp: value.epoch_start_timestamp,
        }
    }
}

impl ClusterHistoryEntry {
    const NUM_FIELDS: u8 = 3;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str =
        "INSERT INTO cluster_history_entries (epoch,total_blocks,epoch_start_timestamp) VALUES ";

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
            separated.push_bind(i32::from(record.epoch));
            separated.push_bind(BigDecimal::from(record.total_blocks));
            separated.push_bind(BigDecimal::from(record.epoch_start_timestamp));

            separated.push_unseparated(") ");

            if num_records >= Self::INSERT_CHUNK_SIZE {
                query_builder.push(" ON CONFLICT (epoch) DO NOTHING");
                let query = query_builder.build();
                query.execute(db_connection).await?;
                num_records = 0;
                query_builder = QueryBuilder::new(Self::INSERT_QUERY);
            }
        }

        if num_records > 0 {
            query_builder.push(" ON CONFLICT (epoch) DO NOTHING");
            let query = query_builder.build();
            query.execute(db_connection).await?;
        }
        Ok(())
    }

    pub async fn fetch_all(db_connection: &Pool<Postgres>) -> Result<Vec<Self>, SqlxError> {
        sqlx::query_as::<_, Self>("SELECT * FROM cluster_history_entries")
            .fetch_all(db_connection)
            .await
    }

    pub fn into_jito_cluster_history_entry(self) -> JitoClusterHistoryEntry {
        JitoClusterHistoryEntry {
            total_blocks: self.total_blocks,
            epoch: self.epoch,
            padding0: [0u8; 2],
            epoch_start_timestamp: self.epoch_start_timestamp,
            padding: [0u8; 240],
        }
    }
}
