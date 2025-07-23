use std::cmp::Ordering;

use crate::{big_decimal_u64::BigDecimalU64, cluster_history_entry::ClusterHistoryEntry};
use sqlx::{
    Error as SqlxError, Pool, Postgres, postgres::PgQueryResult, prelude::FromRow,
    types::BigDecimal,
};
use validator_history::{CircBufCluster, ClusterHistory as JitoClusterHistory};

#[derive(FromRow)]
pub struct ClusterHistory {
    #[sqlx(try_from = "BigDecimalU64")]
    pub struct_version: u64,
    #[sqlx(try_from = "i16")]
    pub bump: u8,
    #[sqlx(try_from = "BigDecimalU64")]
    pub cluster_history_last_update_slot: u64,
}

impl From<JitoClusterHistory> for ClusterHistory {
    fn from(value: JitoClusterHistory) -> Self {
        Self {
            struct_version: value.struct_version,
            bump: value.bump,
            cluster_history_last_update_slot: value.cluster_history_last_update_slot,
        }
    }
}

impl ClusterHistory {
    pub async fn upsert(
        db_connection: &Pool<Postgres>,
        record: Self,
    ) -> Result<PgQueryResult, SqlxError> {
        let sql = "
    INSERT INTO cluster_histories (id,struct_version,bump,cluster_history_last_update_slot) VALUES ($1, $2, $3, $4) \
    ON CONFLICT (id) DO UPDATE SET \
    struct_version = EXCLUDED.struct_version,
    bump = EXCLUDED.bump,
    cluster_history_last_update_slot = EXCLUDED.cluster_history_last_update_slot
    ";
        sqlx::query(sql)
            .bind(1)
            .bind(BigDecimal::from(record.struct_version))
            .bind(i16::from(record.bump))
            .bind(BigDecimal::from(record.cluster_history_last_update_slot))
            .execute(db_connection)
            .await
    }

    pub async fn fetch(db_connection: &Pool<Postgres>) -> Result<Self, SqlxError> {
        sqlx::query_as::<_, Self>(&format!("SELECT * FROM cluster_histories WHERE id = 1",))
            .fetch_one(db_connection)
            .await
    }

    pub fn convert_to_jito_cluster_history(
        self,
        entries: Vec<ClusterHistoryEntry>,
    ) -> JitoClusterHistory {
        let mut entries = entries;
        let mut cluster_history = JitoClusterHistory {
            struct_version: self.struct_version,
            bump: self.bump,
            _padding0: [0u8; 7],
            cluster_history_last_update_slot: self.cluster_history_last_update_slot,
            _padding1: [0u8; 232],
            history: CircBufCluster::default(),
        };

        // Sort entries by epoch, low to high
        entries.sort_by(|a, b| a.epoch.cmp(&b.epoch));
        // Loop through sorted entries insert into ClusterHistory
        for entry in entries.into_iter() {
            if let Some(last_entry) = cluster_history.history.last_mut() {
                match last_entry.epoch.cmp(&entry.epoch) {
                    Ordering::Equal => {
                        *last_entry = entry.into();
                    }
                    Ordering::Greater => {
                        *last_entry = entry.into();
                    }
                    Ordering::Less => {
                        cluster_history
                            .history
                            .push(entry.into());
                    }
                }
            }
        }

        cluster_history
    }
}
