#[macro_export]
macro_rules! decode_db {
    ($field:expr, $field_name:expr) => {
        $field.try_into().map_err(|_| {
            Error::Decode(Box::new(StakenetSimulatorDbError::DecodeError(
                String::from($field_name),
            )))
        })?
    };
}
