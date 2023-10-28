use tokio;

use crate::epic::FORTNITE_NEW_SWITCH_GAME_CLIENT;

mod config;
mod epic;
mod rest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = config::Configuration::read()?;

    if let Some(_device_auth) = &configuration.device_auth {
    } else {
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
                    epic::exchange_code(&details).await?;
                }
                Err(err) => {
                    eprintln!("Error : {}", err);
                }
            };
        }
    }

    Ok(())
}
