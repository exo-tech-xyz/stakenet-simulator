use solana_sdk::pubkey::Pubkey;


pub struct StakeAccount {
  pubkey: Pubkey, 
  discriminator: u32, 
  vote_pubkey: Pubkey,
  rent_exempt_reserve: u64,
  authorized_staker: Pubkey,
  authorized_withdrawer: Pubkey,
  lockup_unix_timestamp: i64,
  lockup_epoch: u64,
  lockup_custodian: Pubkey,
  delegation_voter_pubkey: Pubkey,
  delegation_stake: u64,
  delegation_activation_epoch: u64,
  delegation_deactivation_epoch: u64,
  delegation_warmup_cooldown_rate: f64,
  credits_observed: u64,
  stake_flags: u8,
}

// TODO: Add bulk insert method