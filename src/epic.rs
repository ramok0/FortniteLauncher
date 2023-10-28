use std::{collections::HashMap, str::FromStr};

use crate::{rest::{self, handle_epic_response, EpicError}, config::DeviceAuth};

#[derive(Debug)]
pub enum AuthentificationType {
    AuthorizationCode = 1 ,
    DeviceCode = 2
}

impl std::str::FromStr for AuthentificationType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "1" => Ok(AuthentificationType::AuthorizationCode),
            "2" => Ok(AuthentificationType::DeviceCode),
            _ => Err("Invalid authentication type"),
        }
    }
}

pub struct Client<'a> {
    pub id: &'a str,
    pub secret: &'a str,
}

pub const FORTNITE_NEW_SWITCH_GAME_CLIENT: Client<'static> = Client {
    id: "98f7e42c2e3a4f86a74eb43fbb41ed39",
    secret: "0a2449a2-001a-451e-afec-3e812901c4d7",
};

#[allow(dead_code)]
pub const LAUNCHER_APP_CLIENT_2: Client<'static> = Client {
    id: "34a02cf8f4414e29b15921876da36f9a",
    secret: "daafbccc737745039dffe53d94fc76cf",
};

pub const FORTNITE_IOS_GAME_CLIENT: Client<'static> = Client {
    id: "3446cd72694c4a4485d81b77adbb2141",
    secret: "9209d4a5e25a457fb9b07489d313b41a",
};

const TOKEN:&'static str = "https://account-public-service-prod.ol.epicgames.com/account/api/oauth/token";

#[derive(serde::Deserialize, Clone)]
pub struct ExchangeCode {
    #[serde(rename = "expiresInSeconds")]
    pub expires_in_seconds: i64,
    pub code: String,
    #[serde(rename = "creatingClientId")]
    pub creating_client_id: String,
}

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
pub struct BasicDetails {
    pub access_token: String,
    pub expires_at: String,
    pub client_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AccountDetails {
    pub access_token: String,
    pub expires_at: String,
    pub refresh_token: String,
    pub refresh_expires_at: String,
    pub account_id: String,
    pub client_id: String,

    #[serde(rename = "displayName")]
    pub display_name: String,
    pub app: String,
    pub in_app_id: String,
    pub device_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct Token {
    pub token:String,
    pub expires_at:String
}

pub trait HasClient {
    fn get_client_id(&self) -> &str;
}

pub trait HasToken {
    fn get_access_token(&self) -> Token;
}

pub trait HasRefreshToken {
    fn get_refresh_token(&self) -> Token;
}

pub trait HasIdentity {
    fn get_display_name(&self) -> &str;

    fn get_account_id(&self) -> &str;
}

impl HasToken for BasicDetails {
    fn get_access_token(&self) -> Token {
        Token{
            token: self.access_token.clone(),
            expires_at: self.expires_at.clone()
        }
    }
}

impl HasToken for AccountDetails {
    fn get_access_token(&self) -> Token {
        Token{
            token: self.access_token.clone(),
            expires_at: self.expires_at.clone()
        }
    }
}

impl HasRefreshToken for AccountDetails {
    fn get_refresh_token(&self) -> Token {
        Token{
            token: self.refresh_token.clone(),
            expires_at: self.refresh_expires_at.clone()
        }
    }
}

impl HasClient for BasicDetails {
    fn get_client_id(&self) -> &str {
        &self.client_id
    }
}

impl HasClient for AccountDetails {
    fn get_client_id(&self) -> &str {
        &self.client_id
    }
}

impl HasIdentity for AccountDetails {
    fn get_account_id(&self) -> &str {
        &self.account_id
    }

    fn get_display_name(&self) -> &str {
        &self.display_name
    }
}

pub async fn client_credentials<'a>(
    client: &Client<'a>,
) -> Result<BasicDetails, Box<dyn std::error::Error>> {
    let mut body = HashMap::new();
    body.insert("grant_type", "client_credentials");

    let response = handle_epic_response(
        rest::CLIENT
            .post(TOKEN)
            .form(&body)
            .basic_auth(client.id, Some(client.secret))
            .send()
            .await?
    )
    .await?;

    let details: BasicDetails = response.json().await?;

    Ok(details)
}

pub async fn create_device_code(
    client_credentials: &BasicDetails,
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


pub async fn login_with_device_code<'a>(device_code:&DeviceCode, client: &Client<'a>) -> Result<BasicDetails, Box<dyn std::error::Error>>
{
    let mut body = HashMap::new();
    body.insert("grant_type", "device_code");
    body.insert("device_code", &device_code.device_code);

    if device_code.client_id != client.id {
        return Err(EpicError::ClientMismatch.into());
    }

    let response = handle_epic_response(
        rest::CLIENT
            .post(TOKEN)
            .form(&body)
            .basic_auth(client.id, Some(client.secret))
            .send()
            .await?
    )
    .await?;

    Ok(response.json::<BasicDetails>().await?)
}

pub async fn exchange_code<'a, T:HasToken>(details:&T) -> Result<ExchangeCode, Box<dyn std::error::Error>>
{
    let response = handle_epic_response(
        rest::CLIENT
        .get("https://account-public-service-prod.ol.epicgames.com/account/api/oauth/exchange")
        .bearer_auth(details.get_access_token().token)
        .send()
        .await?
    ).await?;


    Ok(response.json::<ExchangeCode>().await?)
}

#[allow(dead_code)]
pub async fn exchange_to<'a, T:HasToken + HasIdentity+ for<'de> serde::Deserialize<'de>>(details:&T, exchange_to:&Client<'a>) -> Result<T, Box<dyn std::error::Error>>
{
    let code = exchange_code::<T>(details).await?;

    let mut body = HashMap::new();
    body.insert("grant_type", "exchange_code");
    body.insert("exchange_code", code.code.as_str());

    let response = handle_epic_response(
        rest::CLIENT
        .post(TOKEN)
        .basic_auth(exchange_to.id, Some(exchange_to.secret))
        .form(&body)
        .send()
        .await?
    ).await?;

    Ok(response.json::<T>().await?)
}

pub async fn login_with_authorization_code<'a>(code:&str, client:&Client<'a>) -> Result<AccountDetails, Box<dyn std::error::Error>>
{
    let mut body = HashMap::new();
    body.insert("grant_type", "authorization_code");
    body.insert("code", code);

    let response = handle_epic_response(
        rest::CLIENT
        .post(TOKEN)
        .basic_auth(client.id, Some(client.secret))
        .form(&body)
        .send()
        .await?
    ).await?;


    Ok(response.json::<AccountDetails>().await?)
//    Ok(response.json::<Details>().await?)
}

pub async fn create_device_auth<T>(details:&T) -> Result<DeviceAuth, Box<dyn std::error::Error>>
where T: HasToken + HasIdentity 
{
    let response = handle_epic_response(
        rest::CLIENT
        .post(format!("https://account-public-service-prod.ol.epicgames.com/account/api/public/account/{}/deviceAuth", details.get_account_id()))
        .bearer_auth(details.get_access_token().token).send().await?
    ).await?;

    Ok(response.json::<DeviceAuth>().await?)
}
