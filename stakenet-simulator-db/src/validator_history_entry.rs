use serde::Serialize;
use sqlx::Row;
use sqlx::{
    Error, Pool, Postgres, QueryBuilder,
    types::{BigDecimal, Json},
};
use validator_history::{
    ClientVersion as JitoClientVersion, MerkleRootUploadAuthority,
    ValidatorHistoryEntry as JitoValidatorHistoryEntry,
};

pub struct ValidatorHistoryEntry {
    pub id: String,
    pub vote_pubkey: String,
    pub validator_history_entry: JitoValidatorHistoryEntry,
}

impl ValidatorHistoryEntry {
    pub fn new(vote_pubkey: String, validator_history_entry: JitoValidatorHistoryEntry) -> Self {
        Self {
            id: format!("{}-{}", validator_history_entry.epoch, vote_pubkey),
            vote_pubkey,
            validator_history_entry,
        }
    }
}

impl ValidatorHistoryEntry {
    const NUM_FIELDS: u8 = 21;
    // Based on the bind limit of postgres
    const INSERT_CHUNK_SIZE: usize = 65534 / Self::NUM_FIELDS as usize;
    const INSERT_QUERY: &str = "INSERT INTO validator_history_entries (id,vote_pubkey,activated_stake_lamports,epoch,mev_commission,epoch_credits,commission,client_type,version,ip,merkle_root_upload_authority,is_superminority,rank,vote_account_last_update_slot,mev_earned,priority_fee_commission,priority_fee_tips,total_priority_fees,total_leader_slots,blocks_produced,block_data_updated_at_slot) VALUES ";

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
            separated.push_bind(BigDecimal::from(
                record.validator_history_entry.activated_stake_lamports,
            ));
            separated.push_bind(i32::from(record.validator_history_entry.epoch));
            separated
                .push_bind(i32::try_from(record.validator_history_entry.mev_commission).unwrap());
            separated
                .push_bind(i64::try_from(record.validator_history_entry.epoch_credits).unwrap());
            separated.push_bind(i32::from(record.validator_history_entry.commission));
            separated.push_bind(i16::from(record.validator_history_entry.client_type));
            let version: ClientVersion = record.validator_history_entry.version.into();
            separated.push_bind(Json(version));
            separated.push_bind(
                record
                    .validator_history_entry
                    .ip
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join("."),
            );

            separated.push_bind(i16::from(
                record.validator_history_entry.merkle_root_upload_authority as u8,
            ));
            separated.push_bind(i16::from(record.validator_history_entry.is_superminority));
            separated.push_bind(i64::try_from(record.validator_history_entry.rank).unwrap());
            separated.push_bind(BigDecimal::from(
                record.validator_history_entry.vote_account_last_update_slot,
            ));
            separated.push_bind(BigDecimal::from(record.validator_history_entry.mev_earned));
            separated.push_bind(
                i32::try_from(record.validator_history_entry.priority_fee_commission).unwrap(),
            );
            separated.push_bind(BigDecimal::from(
                record.validator_history_entry.priority_fee_tips,
            ));
            separated.push_bind(BigDecimal::from(
                record.validator_history_entry.total_priority_fees,
            ));
            separated.push_bind(
                i64::try_from(record.validator_history_entry.total_leader_slots).unwrap(),
            );
            separated
                .push_bind(i64::try_from(record.validator_history_entry.blocks_produced).unwrap());
            separated.push_bind(BigDecimal::from(
                record.validator_history_entry.block_data_updated_at_slot,
            ));

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

#[derive(Serialize)]
pub struct ClientVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}
impl From<JitoClientVersion> for ClientVersion {
    fn from(value: JitoClientVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

fn from_i32(value: i32) -> MerkleRootUploadAuthority {
    match value {
        1 => MerkleRootUploadAuthority::Other,
        2 => MerkleRootUploadAuthority::OldJitoLabs,
        3 => MerkleRootUploadAuthority::TipRouter,
        _ => MerkleRootUploadAuthority::Unset,
    }
}

fn try_from_i32_to_u8(value: i32) -> Result<u8, sqlx::Error> {
    u8::try_from(value).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

fn try_from_i32_to_u16(value: i32) -> Result<u16, sqlx::Error> {
    u16::try_from(value).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

fn try_from_i32_to_u32(value: i32) -> Result<u32, sqlx::Error> {
    u32::try_from(value).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

fn try_from_i64_to_u64(value: i64) -> Result<u64, sqlx::Error> {
    u64::try_from(value).map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

const FETCH_QUERY: &str = "SELECT id,vote_pubkey,activated_stake_lamports,epoch,mev_commission,epoch_credits,commission,client_type,version,ip,merkle_root_upload_authority,is_superminority,rank,vote_account_last_update_slot,mev_earned,priority_fee_commission,priority_fee_tips,total_priority_fees,total_leader_slots,blocks_produced,block_data_updated_at_slot FROM validator_history_entries;";

pub async fn fetch_all_validator_history_entries(
    db_connection: &Pool<Postgres>,
) -> Result<Vec<ValidatorHistoryEntry>, Error> {
    let rows = sqlx::query(FETCH_QUERY).fetch_all(db_connection).await?;

    let mut history_entries = Vec::new();

    for row in rows {
        let entry = ValidatorHistoryEntry {
            id: row.get("id"),
            vote_pubkey: row.get("vote_pubkey"),
            validator_history_entry: JitoValidatorHistoryEntry {
                activated_stake_lamports: try_from_i64_to_u64(
                    row.get::<i64, _>("activated_stake_lamports"),
                )?,
                epoch: try_from_i32_to_u16(row.get::<i32, _>("epoch"))?,
                mev_commission: try_from_i32_to_u16(row.get::<i32, _>("mev_commission"))?,
                epoch_credits: try_from_i32_to_u32(row.get::<i32, _>("epoch_credits"))?,
                commission: try_from_i32_to_u8(row.get::<i32, _>("commission"))?,
                client_type: try_from_i32_to_u8(row.get::<i32, _>("client_type"))?,
                version: JitoClientVersion {
                    major: try_from_i32_to_u8(row.get::<i32, _>("version_major"))?,
                    minor: try_from_i32_to_u8(row.get::<i32, _>("version_minor"))?,
                    patch: try_from_i32_to_u16(row.get::<i32, _>("version_patch"))?,
                },
                ip: row.get("ip"),
                merkle_root_upload_authority: from_i32(row.get("merkle_root_upload_authority")),
                is_superminority: try_from_i32_to_u8(row.get::<i32, _>("is_superminority"))?,
                rank: try_from_i32_to_u32(row.get::<i32, _>("rank"))?,
                vote_account_last_update_slot: try_from_i64_to_u64(
                    row.get::<i64, _>("vote_account_last_update_slot"),
                )?,
                mev_earned: try_from_i32_to_u32(row.get::<i32, _>("mev_earned"))?,
                priority_fee_commission: try_from_i32_to_u16(
                    row.get::<i32, _>("priority_fee_commission"),
                )?,
                padding0: row.get("padding0"),
                priority_fee_tips: try_from_i64_to_u64(row.get::<i64, _>("priority_fee_tips"))?,
                total_priority_fees: try_from_i64_to_u64(row.get::<i64, _>("total_priority_fees"))?,
                total_leader_slots: try_from_i32_to_u32(row.get::<i32, _>("total_leader_slots"))?,
                blocks_produced: try_from_i32_to_u32(row.get::<i32, _>("blocks_produced"))?,
                block_data_updated_at_slot: try_from_i64_to_u64(
                    row.get::<i64, _>("block_data_updated_at_slot"),
                )?,
                priority_fee_merkle_root_upload_authority: from_i32(
                    row.get::<i32, _>("merkle_root_upload_authority"),
                ),
                padding1: row.get("padding1"),
            },
        };
        history_entries.push(entry);
    }
    Ok(history_entries)
}
