use sqlx::{Error as SqlxError, Pool, Postgres, postgres::PgQueryResult, types::BigDecimal};
use validator_history::ClusterHistory as JitoClusterHistory;

pub struct ClusterHistory {
    pub struct_version: u64,
    pub bump: u8,
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
}
