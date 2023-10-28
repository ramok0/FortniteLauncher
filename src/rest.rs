use reqwest::Response;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLIENT:reqwest::Client = {
        reqwest::Client::builder().build().unwrap()
    };
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EpicErrorDescriptor {
    #[serde(rename = "errorCode")]
    pub error_code: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
    #[serde(rename = "numericErrorCode")]
    pub numeric_error_code: i64,
    #[serde(rename = "originatingService")]
    pub originating_service: String,
    pub intent: String,
    pub message_vars:Option<Vec<String>>
}


#[derive(Debug)]
pub enum EpicError {
    NotFound,
    RateLimited,
    Unauthorized,
    Forbidden,
    InternalError,
    ClientMismatch,
    Expired,
    Other,
    Unknown(EpicErrorDescriptor)
}

impl std::fmt::Display for EpicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self {
            EpicError::NotFound => write!(f, "Not Found")?,
            EpicError::RateLimited => write!(f, "Ratelimited")?,
            EpicError::Unauthorized => write!(f, "Unauthorized")?,
            EpicError::Forbidden => write!(f, "Forbidden")?,
            EpicError::InternalError => write!(f, "Unreachable server.")?,
            EpicError::ClientMismatch => write!(f, "You are using the wrong client")?,
            EpicError::Other => write!(f, "Internal Error")?,
            EpicError::Expired => write!(f, "Expired")?,
            EpicError::Unknown(data) => {
                write!(f, "Error code : {}\nError Messsage : {}\nNumeric error code : {}\nIntent : {}\n", data.error_code, data.error_message, data.numeric_error_code, data.intent)?;
                if let Some(vars) = &data.message_vars {
                    write!(f, "Message vars : \n{}\n", vars.join("\n"))?;
                }
            }
        };

        Ok(())
    }
}

impl std::error::Error for EpicError {}

pub async fn handle_epic_response(response:Response) -> Result<Response, EpicError> {
    let status_code = response.status().as_u16();

    match status_code {
        404 => return Err(EpicError::NotFound),
        403 => {
            println!("Body : {}", response.text().await.unwrap());
            return Err(EpicError::Forbidden)
        },
        429 => return Err(EpicError::RateLimited),
        401 => return Err(EpicError::Unauthorized),
        _ => ()
    }

    if response.status().is_server_error() {
        return Err(EpicError::InternalError);
    }

    if !response.status().is_success() {
        match response.json::<EpicErrorDescriptor>().await {
            Ok(data) => {
                return Err(EpicError::Unknown(data))
            },
            Err(_err) => {
                //TODO : parse epic known errors codes

                return Err(EpicError::Other)
            }
        }
    }
    
    Ok(response)
}
