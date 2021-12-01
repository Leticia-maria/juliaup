use crate::config_file::{load_config_db, save_config_db};
use anyhow::{bail, Context, Result};

pub fn run_command_config(property: String, value: String) -> Result<()> {
    let mut config_data =
        load_config_db().with_context(|| "`config` command failed to load configuration file.")?;

    match property.as_str() {
        "symlinks" => config_data.create_symlinks = match value.as_str() {
            "on"  => true,
            "off" => false,
            _     => bail!("Value for 'symlinks' must be either 'on' or 'off'."),
        },
        s => bail!(format!("Unknown property '{}'.", s)),
    };

    save_config_db(&config_data)?;

    eprintln!("Property '{}' set to '{}'", property, value);

    Ok(())
}
