use std::{collections::HashMap, hash::Hash};

use crate::rest::{self, handle_epic_response, EpicError};

pub struct Client<'a> {
    pub id: &'a str,
    pub secret: &'a str,
}

pub const FORTNITE_NEW_SWITCH_GAME_CLIENT: Client<'static> = Client {
    id: "98f7e42c2e3a4f86a74eb43fbb41ed39",
    secret: "0a2449a2-001a-451e-afec-3e812901c4d7",
};

pub const LAUNCHER_APP_CLIENT_2: Client<'static> = Client {
    id: "34a02cf8f4414e29b15921876da36f9a",
    secret: "daafbccc737745039dffe53d94fc76cf",
};

#[derive(serde::Deserialize, Clone)]
pub struct DeviceCode {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: i32,
    pub interval: i32,
    pub client_id: String,
    #[serde(skip)]
    pub expired:bool
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Details {
    pub access_token: String,
    pub expires_in: i64,
    pub expires_at: String,
    pub token_type: String,
    pub client_id: String,
    pub internal_client: bool,
    pub client_service: String,
    pub product_id: String,
    pub application_id: String,
}

pub async fn client_credentials<'a>(
    client: &Client<'a>,
) -> Result<Details, Box<dyn std::error::Error>> {
    let mut body = HashMap::new();
    body.insert("grant_type", "client_credentials");

    let response = handle_epic_response(
        rest::CLIENT
            .post("https://account-public-service-prod.ol.epicgames.com/account/api/oauth/token")
            .form(&body)
            .basic_auth(client.id, Some(client.secret))
            .send()
            .await?
    )
    .await?;

    let details: Details = response.json().await?;

    Ok(details)
}

pub async fn create_device_code(
    client_credentials: &Details,
) -> Result<DeviceCode, Box<dyn std::error::Error>> {

    let mut body = HashMap::new();
    body.insert("prompt", "login");

    let response = handle_epic_response(
        rest::CLIENT.
        post("https://account-public-service-prod.ol.epicgames.com/account/api/oauth/deviceAuthorization")
        .form(&body)
        .bearer_auth(&client_credentials.access_token).send().await?
    ).await?;

    Ok(response.json::<DeviceCode>().await?)
}


pub async fn login_with_device_code<'a>(details:&DeviceCode, client: &Client<'a>) -> Result<Details, Box<dyn std::error::Error>>
{
    let mut body = HashMap::new();
    body.insert("grant_type", "device_code");
    body.insert("device_code", &details.device_code);

    if details.client_id != client.id {
        return Err(EpicError::ClientMismatch.into());
    }

    let response = handle_epic_response(
        rest::CLIENT
            .post("https://account-public-service-prod.ol.epicgames.com/account/api/oauth/token")
            .form(&body)
            .basic_auth(client.id, Some(client.secret))
            .send()
            .await?
    )
    .await?;

    Ok(response.json::<Details>().await?)
}

pub async fn exchange_code<'a>(details:&Details) -> Result<(), Box<dyn std::error::Error>>
{
    let response = handle_epic_response(
        rest::CLIENT
        .get("https://account-public-service-prod.ol.epicgames.com/account/api/oauth/exchange")
        .bearer_auth(&details.access_token)
        .send()
        .await?
    ).await?;

    println!("Exchange code : {}", response.text().await?);

    Ok(())
}

// pub async fn exchange_to<'a>(details:&Details, exchange_to:&Client<'a>) -> Result<Details, Box<dyn std::error::Error>>
// {

// }
