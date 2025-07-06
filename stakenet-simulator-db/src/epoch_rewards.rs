use sqlx::{Error, FromRow, Pool, Postgres, QueryBuilder, types::BigDecimal};

#[derive(FromRow)]
pub struct EpochRewards {
    pub id: String,
    pub vote_pubkey: String,
    pub epoch: u64,
    pub inflation_commission_bps: u16,
    #[sqlx(try_from = "i64")]
    pub total_inflation_rewards: u64,
    pub mev_commission_bps: u16,
    #[sqlx(try_from = "i64")]
    pub total_mev_rewards: u64,
    pub priority_fee_commission_bps: u16,
    #[sqlx(try_from = "i64")]
    pub total_priority_fee_rewards: u64,
    #[sqlx(try_from = "i64")]
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
}
