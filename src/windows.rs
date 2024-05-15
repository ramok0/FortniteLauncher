use windows::Win32::{Foundation::{CloseHandle, HANDLE}, System::{ProcessStatus::{EnumProcesses, GetProcessImageFileNameA}, Threading::{OpenProcess, PROCESS_QUERY_INFORMATION}}};



pub unsafe fn find_process(name:&str) -> Option<u32> {
    let mut process_ids = vec![0u32; 1024];
    let mut cb_needed = 0;
    if 
        EnumProcesses(
            process_ids.as_mut_ptr(),
            (process_ids.len() as u32 * std::mem::size_of::<u32>() as u32) as u32,
            &mut cb_needed,
        )
    .is_ok() {
        let number_of_processes = cb_needed as usize / std::mem::size_of::<u32>();
        for i in 0..number_of_processes {
            let pid = process_ids[i];
            let handle = match OpenProcess(PROCESS_QUERY_INFORMATION, false, pid).ok() {
                Some(handle) => handle,
                None => continue,
            };

            let mut buffer = [0u8; 1024];

            if GetProcessImageFileNameA(handle, &mut buffer) == 0 {
                CloseHandle(handle);
                continue;
            }

            let process_name = String::from_utf8_lossy(&buffer);
            if process_name.contains(name) {
                CloseHandle(handle);
                return Some(pid);
            }

            CloseHandle(handle);
        }
    }

    None
} 