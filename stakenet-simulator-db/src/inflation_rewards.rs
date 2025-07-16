use solana_client::rpc_response::RpcInflationReward;
use solana_sdk::pubkey::Pubkey;
use sqlx::{Error as SqlxError, Pool, Postgres, QueryBuilder, types::BigDecimal};

pub struct InflationReward {
    pub id: String,
    pub stake_account: String,
    pub epoch: u64,
    pub effective_slot: u64,
    pub amount: u64,
    pub post_balance: u64,
    pub commission: Option<u8>,
}

impl InflationReward {
    const NUM_FIELDS: u8 = 7;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO inflation_rewards (id, stake_account,epoch,effective_slot,amount,post_balance,commission) VALUES ";

    pub fn from_rpc_inflation_reward(
        rpc_inflation_reward: RpcInflationReward,
        stake_account: &Pubkey,
    ) -> Self {
        let id = format!("{}-{}", rpc_inflation_reward.epoch, stake_account);
        Self {
            id,
            stake_account: stake_account.to_string(),
            epoch: rpc_inflation_reward.epoch,
            effective_slot: rpc_inflation_reward.effective_slot,
            amount: rpc_inflation_reward.amount,
            post_balance: rpc_inflation_reward.post_balance,
            commission: rpc_inflation_reward.commission,
        }
    }

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
            separated.push_bind(record.id);
            separated.push_bind(record.stake_account);
            separated.push_bind(BigDecimal::from(record.epoch));
            separated.push_bind(BigDecimal::from(record.effective_slot));
            separated.push_bind(BigDecimal::from(record.amount));
            separated.push_bind(BigDecimal::from(record.post_balance));
            separated.push_bind(record.commission.map(|x| i16::from(x)));

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
}
