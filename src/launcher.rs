use std::{
    ffi::{CString, OsString},
    os::windows::prelude::OsStringExt,
    path::PathBuf,
    process::{Child, Command},
};

use winapi::um::{
    fileapi::GetLogicalDriveStringsW,
    handleapi::CloseHandle,
    processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA},
    winbase::{NORMAL_PRIORITY_CLASS, CREATE_SUSPENDED}
};

use crate::epic::{AntiCheatProvider, ExchangeCode, HasIdentity, HasToken};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LauncherInstalled {
    #[serde(rename = "InstallationList")]
    pub installation_list: Vec<InstallEntry>,
}

impl LauncherInstalled {
    pub fn find(&self, artifact_id: &str) -> Option<&InstallEntry> {
        self.installation_list
            .iter()
            .find(|entry| entry.artifact_id == artifact_id)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InstallEntry {
    #[serde(rename = "InstallLocation")]
    pub install_location: String,
    #[serde(rename = "NamespaceId")]
    pub namespace_id: String,
    #[serde(rename = "ItemId")]
    pub item_id: String,
    #[serde(rename = "ArtifactId")]
    pub artifact_id: String,
    #[serde(rename = "AppVersion")]
    pub app_version: String,
    #[serde(rename = "AppName")]
    pub app_name: String,
}

pub fn get_launcher_installed() -> Result<LauncherInstalled, Box<dyn std::error::Error>> {
    let mut buffer: [u16; 255] = unsafe { std::mem::zeroed() };

    let len = unsafe { GetLogicalDriveStringsW(255, buffer.as_mut_ptr()) };

    let drives = OsString::from_wide(&buffer[0..len as usize])
        .to_string_lossy()
        .into_owned();

    let letters: Vec<&str> = drives.split('\0').collect();

    let result = letters
        .iter()
        .filter_map(|x| {
            let mut buf = PathBuf::from(x);
            buf.push("ProgramData\\Epic\\UnrealEngineLauncher\\LauncherInstalled.dat");

            if buf.exists() {
                Some(buf)
            } else {
                None
            }
        })
        .next();

    if let Some(launcher_installed_path) = result {
        let content = std::fs::read_to_string(launcher_installed_path)?;

        return Ok(serde_json::from_str::<LauncherInstalled>(&content)?);
    } else {
        return Err("Installation Path not found".into());
    }
}

const CURRENT_OBFUSCATION_ID: &str = "ts8I5xx-YxQxC3swakWvk_NTAzp5XA";
pub fn generate_arguments<'a, T>(
    details: &'a T,
    exchange_code: &'a ExchangeCode,
    caldera: &'a AntiCheatProvider,
) -> Vec<String>
where
    T: HasIdentity + HasToken,
{
    let mut params: Vec<(&str, Option<&str>)> = Vec::new();

    params.push(("obfuscationid", Some(CURRENT_OBFUSCATION_ID)));
    params.push(("AUTH_LOGIN", Some("unused")));
    params.push(("AUTH_PASSWORD", Some(&exchange_code.code)));
    params.push(("AUTH_TYPE", Some("exchangecode")));
    params.push(("epicapp", Some("Fortnite")));
    params.push(("epicenv", Some("Prod")));
    params.push(("EpicPortal", None));
    params.push(("epicusername", Some(details.get_display_name())));
    params.push(("epicuserid", Some(details.get_account_id())));
    params.push(("epiclocale", Some("en")));
    params.push(("epicsandboxid", Some("fn")));
    match caldera.provider.as_str() {
        "BattlEye" => {
            params.push(("noeac", None));
            params.push(("noeaceos", None));
            params.push(("fromfl", Some("be")));
        }
        "EasyAntiCheatEOS" => {
            params.push(("noeac", None));
            params.push(("nobe", None));
            params.push(("fromfl", Some("eaceos")));
        }
        "EasyAntiCheat" => {
            params.push(("noeaceos", None));
            params.push(("nobe", None));
            params.push(("fromfl", Some("eac")));
        }
        _ => todo!(),
    }
    params.push(("caldera", Some(&caldera.jwt)));

    // let _result = params
    //     .iter()
    //     .map(|arg| {
    //         if let Some(value) = arg.1 {
    //             format!("-{}={}", arg.0, value)
    //         } else {
    //             format!("-{}", arg.0)
    //         }
    //     })
    //     .collect::<Vec<String>>()
    //     .join(" ");

        let result = params
        .iter()
        .map(|arg| {
            if let Some(value) = arg.1 {
                format!("-{}={}", arg.0, value)
            } else {
                format!("-{}", arg.0)
            }
        })
        .collect::<Vec<String>>();

    result
}

// pub unsafe fn suspend_process(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
//     let hProcess = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);

//     if hProcess == INVALID_HANDLE_VALUE {
//         return Err("Failed to create a handle".into());
//     }

//     let ntdll = GetModuleHandleA(CString::new("ntdll.dll")?.as_ptr());
//     if ntdll.is_null() {
//         return Err("Failed to get NTDLL address".into());
//     }

//     let function_name = CString::new("NtSuspendProcess")?;

//     let function_ptr = GetProcAddress(ntdll, function_name.as_ptr());

//     if function_ptr.is_null() {
//         return Err("Failed to get NtSuspendProcess address".into());
//     }

//     type NtSuspendProcessFn =
//         unsafe extern "system" fn(process_handle: winapi::um::winnt::HANDLE) -> u32;
//     let nt_suspend_process: NtSuspendProcessFn = unsafe { std::mem::transmute(function_ptr) };

//     let nt_status = unsafe { nt_suspend_process(hProcess) };

//     if nt_status == 0x0 {
//         return Ok(());
//     } else {
//         return Err(format!("Error : {}", nt_status).into());
//     }
// }

pub fn spawn_process(
    path: &str,
    arguments: Option<Vec<String>>,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mut command = Command::new(&path);

    if let Some(args) = arguments {
        command.args(args);
    }

    let child = command.spawn()?;

    Ok(child)
}

pub fn create_process(
    path: &str,
    arguments: Option<String>,
    spawn_suspended: bool,
) -> Result<(), Box<dyn std::error::Error>> {

    let args = match arguments {
        Some(data) => format!("\"{}\" {}", path, data),
        None => format!("\"{}\"", path)
    };

    let c_arguments = CString::new(args)?;

    let mut creation_flags = NORMAL_PRIORITY_CLASS;

    if spawn_suspended {
        creation_flags = creation_flags | CREATE_SUSPENDED;
    }

    let result: Result<(), Box<dyn std::error::Error>> = unsafe {
        let mut startup_info: STARTUPINFOA = std::mem::zeroed();
        let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();

        if CreateProcessA(
            std::ptr::null(),
            c_arguments.as_ptr() as *mut i8,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            creation_flags,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut startup_info as *mut STARTUPINFOA,
            &mut process_info,
        ) != 0
        {
            CloseHandle(process_info.hThread);
            CloseHandle(process_info.hProcess);

            Ok(())
        } else {
            Err("Windows API Error".into())
        }
    };

    Ok(result?)
}
