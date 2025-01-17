#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
use crate::config_file::*;
#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
use crate::utils::get_juliaserver_base_url;
#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
use anyhow::{bail, anyhow};
#[cfg(feature = "selfupdate")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
use crate::operations::{download_juliaup_version,download_extract_sans_parent};
#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
use crate::get_juliaup_target;
#[cfg(feature = "windowsstore")]
use windows::{core::Interface,Win32::{System::Console::GetConsoleWindow, UI::Shell::IInitializeWithWindow}};

#[cfg(not(feature = "selfupdate"))]
pub fn run_command_selfupdate() -> Result<()> {
    println!("error: self-update is disabled for this build of juliaup");
    println!("error: you should probably use your system package manager to update juliaup");

    Ok(())
}

#[cfg(feature = "selfupdate")]
#[cfg(not(feature = "windowsstore"))]
pub fn run_command_selfupdate() -> Result<()> {
    let config_data =
        load_config_db().with_context(|| "`selfupdate` command failed to load configuration db.")?;

    let juliaup_channel = match config_data.juliaup_channel {
        Some(juliaup_channel) => juliaup_channel,
        None => "release".to_string()
    };

    let juliaupserver_base = get_juliaserver_base_url()
            .with_context(|| "Failed to get Juliaup server base URL.")?;
            
    let version_url_path = match juliaup_channel.as_str() {
        "release" => "juliaup/RELEASECHANNELVERSION",
        "releasepreview" => "juliaup/RELEASEPREVIEWCHANNELVERSION",
        "dev" => "juliaup/DEVCHANNELVERSION",
        _ => bail!("Juliaup is configured to a channel named '{}' that does not exist.", &juliaup_channel)
    };

    let version_url = juliaupserver_base.join(version_url_path)
        .with_context(|| format!("Failed to construct a valid url from '{}' and '{}'.", juliaupserver_base, version_url_path))?;

    let version = download_juliaup_version(&version_url.to_string())?;

    let juliaup_target = get_juliaup_target();

    let juliaupserver_base = get_juliaserver_base_url()
            .with_context(|| "Failed to get Juliaup server base URL.")?;

    let download_url_path = format!("juliaup/bin/juliaup-{}-{}.tar.gz", version, juliaup_target);

    let new_juliaup_url = juliaupserver_base.join(&download_url_path)
            .with_context(|| format!("Failed to construct a valid url from '{}' and '{}'.", juliaupserver_base, download_url_path))?;

    let my_own_path = std::env::current_exe()
        .with_context(|| "Could not determine the path of the running exe.")?;

    let my_own_folder = my_own_path.parent()
        .ok_or_else(|| anyhow!("Could not determine parent."))?;

    println!("We are on the juliaup channel '{}'.", juliaup_channel);
    println!("The version URL is {}.", version_url);
    println!("The current version is {}.", version);
    println!("We will replace {:?}.", my_own_path);
    println!("We will replace files in {:?}.", my_own_folder);
    println!("We are on {}.", juliaup_target);
    println!("We will download from {}.", new_juliaup_url);

    download_extract_sans_parent(&new_juliaup_url.to_string(), &my_own_folder, 0)?;

    Ok(())
}

#[cfg(feature = "selfupdate")]
#[cfg(feature = "windowsstore")]
pub fn run_command_selfupdate() -> Result<()> {
    let update_manager = windows::Services::Store::StoreContext::GetDefault()    
        .with_context(|| "Failed to get the store context.")?;

    let interop: IInitializeWithWindow = update_manager.cast()
        .with_context(|| "Failed to cast the store context to IInitializeWithWindow.")?;

    unsafe {
        let x = GetConsoleWindow();

        interop.Initialize(x)
            .with_context(|| "Call to IInitializeWithWindow.Initialize failed.")?;
    }
    
    let updates = update_manager.GetAppAndOptionalStorePackageUpdatesAsync()
        .with_context(|| "Call to GetAppAndOptionalStorePackageUpdatesAsync failed.")?
        .get()
        .with_context(|| "get on the return of GetAppAndOptionalStorePackageUpdatesAsync failed.")?;

    if updates.Size().with_context(|| "Call to Size on update results failed.")? > 0 {
        println!("An update is available.");

        let download_operation = update_manager.RequestDownloadAndInstallStorePackageUpdatesAsync(updates)
            .with_context(|| "Call to RequestDownloadAndInstallStorePackageUpdatesAsync failed.")?;

        download_operation.get()
            .with_context(|| "get on result from RequestDownloadAndInstallStorePackageUpdatesAsync failed.")?;
        // This code will not be reached if the user opts to install updates
    } else {
        println!("No no updates available.");
    }

    Ok(())
}