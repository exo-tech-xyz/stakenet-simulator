use num_bigint::{BigInt, ToBigInt};
use num_traits::identities::Zero;
use sqlx::{
    Decode, FromRow, Postgres, Type, ValueRef,
    postgres::{PgTypeInfo, PgValueRef},
    types::BigDecimal,
};

#[derive(Clone, Debug, FromRow)]
pub struct BigDecimalU64(pub u64);

impl From<BigDecimalU64> for u64 {
    fn from(value: BigDecimalU64) -> Self {
        value.0
    }
}

// Implement sqlx::Type for OptionU64
impl Type<Postgres> for BigDecimalU64 {
    fn type_info() -> PgTypeInfo {
        // BIGINT in PostgreSQL
        PgTypeInfo::with_name("NUMERIC(20,0)")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("BIGINT") || *ty == PgTypeInfo::with_name("NUMERIC")
    }
}

// Implement sqlx::Decode for OptionU64
impl<'r> Decode<'r, Postgres> for BigDecimalU64 {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Handle NULL case
        if value.is_null() {
            return Err("Cannot convert null to BigDecimalU64".into());
        }

        // Decode as BigDecimal and convert to u64
        let val: BigDecimal = Decode::<Postgres>::decode(value)?;
        if val.lt(&BigDecimal::new(BigInt::zero(), 0)) {
            return Err("Cannot convert negative BigDecimal to u64".into());
        }
        let bigint: BigInt = val.to_bigint().unwrap();

        // Check for overflow
        if bigint > BigInt::from(u64::MAX) {
            return Err("too big for u64".into());
        }

        Ok(BigDecimalU64(bigint.try_into()?))
    }
}
