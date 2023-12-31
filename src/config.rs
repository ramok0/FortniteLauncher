#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct DeviceAuth {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub secret: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Configuration {
    pub device_auth: Option<DeviceAuth>,
    pub fortnite_path: Option<String>
}

impl Configuration {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let path_buf = std::path::PathBuf::from("config.json");
        if path_buf.exists() {
            let data_str = std::fs::read_to_string(path_buf)?;
            let data: Configuration = serde_json::from_str(&data_str)?;

            Ok(data)
        } else {
            //create default config
            let data = Self::default();
            data.flush()?;

            Ok(data)
        }
    }

    pub fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write("config.json", serde_json::to_string_pretty(&self)?)?;
        if cfg!(debug_assertions){
            println!("Flushed configuration successfully !");
        }
        Ok(())
    }
}

impl Drop for Configuration {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self { device_auth: None, fortnite_path: None }
    }
}
