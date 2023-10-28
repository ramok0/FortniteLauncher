use std::io::Write;

use config::Configuration;
use epic::AccountDetails;
use tokio;

use crate::{epic::{FORTNITE_NEW_SWITCH_GAME_CLIENT, AuthentificationType, LAUNCHER_APP_CLIENT_2, FORTNITE_IOS_GAME_CLIENT, HasIdentity}, rest::EpicError};

mod config;
mod epic;
mod rest;

async fn onboarding_authorization_code(configuration:&mut Configuration) -> Result<AccountDetails, Box<dyn std::error::Error>> {
    print!("\nGet your device code here : https://www.epicgames.com/id/api/redirect?clientId={}&responseType=code\nAuthorization code : ", FORTNITE_IOS_GAME_CLIENT.id);
    std::io::stdout().flush()?;

    let mut authorization_code = String::new();

    std::io::stdin().read_line(&mut authorization_code)?;

    if authorization_code.len() > 32 {
        authorization_code = authorization_code[0..32].to_string();
    }

    let details = epic::login_with_authorization_code(authorization_code.as_str(), &FORTNITE_IOS_GAME_CLIENT).await?;
    
   // let details = epic::exchange_to(&details, &LAUNCHER_APP_CLIENT_2).await?;

    let device_auth = epic::create_device_auth(&details).await?;

    configuration.device_auth = Some(device_auth);

    Ok(details)
}

async fn onboarding_device_code(configuration:&mut Configuration) -> Result<AccountDetails, Box<dyn std::error::Error>>
{
    let details = epic::client_credentials(&FORTNITE_NEW_SWITCH_GAME_CLIENT).await?;
    let device_code = epic::create_device_code(&details).await?;

    println!("Please go to {} and enter this code : {} to connect to your epicgames account !", device_code.verification_uri, device_code.user_code);

    let number_of_intervals = device_code.expires_in / device_code.interval;

    for _i in 0..number_of_intervals {
        std::thread::sleep(std::time::Duration::from_secs(device_code.interval as u64));

        let _ = match epic::login_with_device_code(
            &device_code,
            &FORTNITE_NEW_SWITCH_GAME_CLIENT,
        )
        .await
        {
            Ok(details) => {
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

async fn epic_login(configuration:&mut Configuration) -> Result<AccountDetails, Box<dyn std::error::Error>>
{
    if let Some(device_auth) = &configuration.device_auth {
        let ios_details = epic::login_with_device_auth(device_auth, &FORTNITE_IOS_GAME_CLIENT).await?;
        let details = epic::exchange_to(&ios_details, &LAUNCHER_APP_CLIENT_2).await?;

        Ok(details)
    } else {
        let mut choice:String = String::new();

        print!("[1] AuthorizationCode\n[2] DeviceCode\n\now do you want to authentificate : ");
        std::io::stdout().flush()?;

        std::io::stdin().read_line(&mut choice)?;

        let auth_type:AuthentificationType = choice.parse()?;

        let details = match auth_type {
            AuthentificationType::AuthorizationCode => {
                let ios_details = onboarding_authorization_code(configuration).await?;
                let details = epic::exchange_to(&ios_details, &LAUNCHER_APP_CLIENT_2).await?;

                details
            //    println!("Access_token : {}", &details.access_token);
            },
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

    

    Ok(())
}
