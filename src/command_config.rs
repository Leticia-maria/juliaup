use crate::config_file::JuliaupConfigChannel;
use crate::config_file::{load_config_db, save_config_db};
use crate::operations::{create_symlink, remove_symlink};
use anyhow::{bail, Context, Result};

pub fn run_command_config(property: String, value: String) -> Result<()> {
    let mut config_data =
        load_config_db().with_context(|| "`config` command failed to load configuration file.")?;

    match property.as_str() {
        "symlinks" => {
            if std::env::consts::OS == "windows" {
                bail!("Symlinks not supported on Windows.");
            }

            config_data.create_symlinks = match value.as_str() {
                "on"  => true,
                "off" => false,
                _     => bail!("Value for 'symlinks' must be either 'on' or 'off'."),
            };

            if config_data.create_symlinks {
                for (channel_name, channel) in &config_data.installed_channels {
                    match channel {
                        JuliaupConfigChannel::SystemChannel { version } =>
                            create_symlink(&version, &format!("julia-{}", channel_name))?,
                        // TODO
                        JuliaupConfigChannel::LinkedChannel { .. } => {},
                    };
                }
            }
            else {
                for (channel_name, channel) in &config_data.installed_channels {
                    match channel {
                        JuliaupConfigChannel::SystemChannel { version: _ } =>
                            remove_symlink(&format!("julia-{}", channel_name))?,
                        // TODO
                        JuliaupConfigChannel::LinkedChannel { .. } => {},
                    };
                }
            }
        },
        s => bail!(format!("Unknown property '{}'.", s)),
    };

    save_config_db(&config_data)?;

    eprintln!("Property '{}' set to '{}'", property, value);

    Ok(())
}
