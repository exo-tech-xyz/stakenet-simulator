use crate::{big_decimal_u64::BigDecimalU64, validator_history_entry::ValidatorHistoryEntry};
use solana_sdk::pubkey::Pubkey;
use sqlx::{Error as SqlxError, Pool, Postgres, QueryBuilder, prelude::FromRow, types::BigDecimal};
use std::{cmp::Ordering, str::FromStr};
use validator_history::{CircBuf, ValidatorHistory as JitoValidatorHistory};

#[derive(FromRow, Clone)]
pub struct ValidatorHistory {
    #[sqlx(try_from = "i64")]
    pub struct_version: u32,
    /// Vote account pubkey
    pub vote_account: String,
    #[sqlx(try_from = "i64")]
    pub index: u32,
    #[sqlx(try_from = "i16")]
    pub bump: u8,
    /// Unix timestamp when the validator last reported its IP address
    #[sqlx(try_from = "BigDecimalU64")]
    pub last_ip_timestamp: u64,
    /// Unix timestamp when the validator last reported its version
    #[sqlx(try_from = "BigDecimalU64")]
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
    /// Number of fields in the validator_histories table
    const NUM_FIELDS: u8 = 6;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO validator_histories (vote_account,struct_version,index,bump,last_ip_timestamp,last_version_timestamp) VALUES ";

    pub async fn bulk_insert(
        db_connection: &Pool<Postgres>,
        records: Vec<Self>,
    ) -> Result<(), SqlxError> {
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
            separated.push_bind(record.vote_account);
            separated.push_bind(i64::from(record.struct_version));
            separated.push_bind(i64::from(record.index));
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

    pub async fn fetch_all(db_connection: &Pool<Postgres>) -> Result<Vec<Self>, SqlxError> {
        sqlx::query_as::<_, Self>("SELECT * FROM validator_histories")
            .fetch_all(db_connection)
            .await
    }

    /// Uses our db table data to convert to a JitoValidatorHistory struct
    pub fn convert_to_jito_validator_history(
        self,
        entries: &mut [ValidatorHistoryEntry],
    ) -> JitoValidatorHistory {
        let mut validator_history = JitoValidatorHistory {
            struct_version: self.struct_version,
            vote_account: Pubkey::from_str(&self.vote_account).unwrap(),
            index: self.index,
            bump: self.bump,
            _padding0: [0u8; 7],
            last_ip_timestamp: self.last_ip_timestamp,
            last_version_timestamp: self.last_ip_timestamp,
            _padding1: [0u8; 232],
            history: CircBuf::default(),
        };

        // Sort entries by epoch, low to high
        entries.sort_by(|a, b| {
            a.validator_history_entry
                .epoch
                .cmp(&b.validator_history_entry.epoch)
        });
        // Loop through sorted entries insert into ValidatorHistory
        for entry in entries.iter_mut() {
            if let Some(last_entry) = validator_history.history.last_mut() {
                match last_entry.epoch.cmp(&entry.validator_history_entry.epoch) {
                    Ordering::Equal => {
                        *last_entry = entry.validator_history_entry;
                    }
                    Ordering::Greater => {
                        *last_entry = entry.validator_history_entry;
                    }
                    Ordering::Less => {
                        validator_history
                            .history
                            .push(entry.validator_history_entry);
                    }
                }
            } else {
                validator_history
                    .history
                    .push(entry.validator_history_entry);
            }
        }

        validator_history
    }
}
