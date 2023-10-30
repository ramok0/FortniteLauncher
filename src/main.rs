use std::{io::Write, path::PathBuf};

use config::Configuration;
//use egmanifest_rs::Parsable;
use epic::AccountDetails;
use tokio;

use crate::{
    epic::{
        AuthentificationType, HasIdentity, FORTNITE_IOS_GAME_CLIENT,
        FORTNITE_NEW_SWITCH_GAME_CLIENT, LAUNCHER_APP_CLIENT_2,
    },
    rest::EpicError,
};

mod config;
mod epic;
mod launcher;
mod rest;

async fn onboarding_authorization_code(
    configuration: &mut Configuration,
) -> Result<AccountDetails, Box<dyn std::error::Error>> {
    print!("\nGet your authorization code here : https://www.epicgames.com/id/api/redirect?clientId={}&responseType=code\nAuthorization code : ", FORTNITE_IOS_GAME_CLIENT.id);
    std::io::stdout().flush()?;

    let mut authorization_code = String::new();

    std::io::stdin().read_line(&mut authorization_code)?;

    if authorization_code.len() > 32 {
        authorization_code = authorization_code[0..32].to_string();
    }

    let details =
        epic::login_with_authorization_code(authorization_code.as_str(), &FORTNITE_IOS_GAME_CLIENT)
            .await?;

    // let details = epic::exchange_to(&details, &LAUNCHER_APP_CLIENT_2).await?;

    let device_auth = epic::create_device_auth(&details).await?;

    configuration.device_auth = Some(device_auth);

    Ok(details)
}

async fn onboarding_device_code(
    configuration: &mut Configuration,
) -> Result<AccountDetails, Box<dyn std::error::Error>> {
    let details = epic::client_credentials(&FORTNITE_NEW_SWITCH_GAME_CLIENT).await?;
    let device_code = epic::create_device_code(&details).await?;

    println!(
        "Please go to {} and enter this code : {} to connect to your epicgames account !",
        device_code.verification_uri, device_code.user_code
    );

    let number_of_intervals = device_code.expires_in / device_code.interval;

    for _i in 0..number_of_intervals {
        std::thread::sleep(std::time::Duration::from_secs(device_code.interval as u64));

        let _ = match epic::login_with_device_code(&device_code, &FORTNITE_NEW_SWITCH_GAME_CLIENT)
            .await
        {
            Ok(details) => {
                let details = epic::exchange_to(&details, &FORTNITE_IOS_GAME_CLIENT).await?;
                let device_auth = epic::create_device_auth(&details).await?;

                configuration.device_auth = Some(device_auth);

                return Ok(details);
            }
            Err(err) => {
                eprintln!("Error : {}", err);
                return Err(err.into());
            }
        };
    }

    return Err(EpicError::Expired.into());
}

async fn epic_login(
    configuration: &mut Configuration,
) -> Result<AccountDetails, Box<dyn std::error::Error>> {
    if let Some(device_auth) = &configuration.device_auth {
        let ios_details =
            epic::login_with_device_auth(device_auth, &FORTNITE_IOS_GAME_CLIENT).await?;
        let details = epic::exchange_to(&ios_details, &LAUNCHER_APP_CLIENT_2).await?;

        Ok(details)
    } else {
        let mut choice: String = String::new();

        print!("[1] AuthorizationCode\n[2] DeviceCode\n\nHow do you want to authentificate : ");
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut choice)?;

        let auth_type: AuthentificationType = choice.parse()?;

        let details = match auth_type {
            AuthentificationType::AuthorizationCode => {
                let ios_details = onboarding_authorization_code(configuration).await?;
                let details = epic::exchange_to(&ios_details, &LAUNCHER_APP_CLIENT_2).await?;

                details
                //    println!("Access_token : {}", &details.access_token);
            }
            AuthentificationType::DeviceCode => {
                let ios_details = onboarding_device_code(configuration).await?;
                let details = epic::exchange_to(&ios_details, &LAUNCHER_APP_CLIENT_2).await?;

                details
            }
        };

        Ok(details)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut configuration = config::Configuration::read()?;

    let details = epic_login(&mut configuration).await?;
    println!("Welcome back, {}", details.get_display_name());

    let anti_cheat = epic::request_anti_cheat_provider(&details).await?;
    println!("AntiCheat Provider : {}", anti_cheat.provider);

    if configuration.fortnite_path.is_none()
        || !PathBuf::from(configuration.fortnite_path.clone().unwrap()).exists()
    {
        if let Some(entry) = launcher::get_launcher_installed()?.find("Fortnite") {
            configuration.fortnite_path = Some(entry.install_location.clone());
        } else {
            return Err("No Fortnite Path has been found, please fill it in config.json.".into());
        }
    }

    let exchange_code = epic::exchange_code(&details).await?;
    if cfg!(debug_assertions) {
        println!("Created exchange code successfully : {}", &exchange_code.code);
    }
    let arguments = launcher::generate_arguments(&details, &exchange_code, &anti_cheat);

    let mut fortnite_binary_folder = std::path::PathBuf::from(configuration.fortnite_path.clone().unwrap());
    fortnite_binary_folder.push("FortniteGame/Binaries/Win64");

    if !fortnite_binary_folder.exists()
    {
        return Err("Fortnite Binary does not exists !".into());
    }

    let mut fortnite_launcher_path_buf = fortnite_binary_folder.clone();
    fortnite_launcher_path_buf.push("FortniteLauncher.exe");

    let mut fortnite_binary_path_buf = fortnite_binary_folder.clone();
    fortnite_binary_path_buf.push("FortniteClient-Win64-Shipping.exe");

    let mut fortnite_anticheat_path_buf = fortnite_binary_folder.clone();
    
    match anti_cheat.provider.as_str() {
        "EasyAntiCheat" => fortnite_anticheat_path_buf.push("FortniteClient-Win64-Shipping_EAC.exe"),
        "EasyAntiCheatEOS" => fortnite_anticheat_path_buf.push("FortniteClient-Win64-Shipping_EAC_EOS.exe"),
        "BattlEye" => fortnite_anticheat_path_buf.push("FortniteClient-Win64-Shipping_BE.exe"),
        _ => ()
    }

    launcher::create_process(fortnite_anticheat_path_buf.to_str().ok_or("Failed to str")?,  None, true)?;
    launcher::create_process(fortnite_launcher_path_buf.to_str().ok_or("Failed to str")?, None, true)?;
    let fortnite_process = launcher::spawn_process(fortnite_binary_path_buf.to_str().ok_or("Failed to str")?, Some(arguments))?;

    if cfg!(debug_assertions)
    {
        println!("Created Fortnite Process, PID : {}", fortnite_process.id());
    }

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
