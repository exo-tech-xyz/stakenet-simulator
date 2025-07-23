#[macro_export]
macro_rules! modify_config_parameter_from_args {
    ($args:expr, $config:expr, $field:ident) => {
        if let Some(value) = $args.$field {
          $config.parameters.$field = value;
        }
    };
}