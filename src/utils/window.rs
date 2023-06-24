use anyhow::{anyhow, Result};
use std::ffi::CString;
use winapi::um::winuser::{
    EnumWindows, FindWindowA, GetWindowTextA, GetWindowTextLengthA, IsWindowVisible,
};

pub fn find_and_focus_window(title: &String) -> Result<()> {
    let cstr = CString::new(title.as_str())
        .map_err(|e| anyhow!("Error:Failed to convert '{title}' to cstring : {e}"))?;
    let find_res = unsafe { FindWindowA(std::ptr::null(), cstr.as_ptr()) };
    if find_res.is_null() {
        return Err(anyhow!("Error:Failed to find a window named '{title}'"));
    } else {
        let title_length = unsafe { GetWindowTextLengthA(find_res) };
        let title_buffer = vec![0u8; (title_length + 1) as usize];
        unsafe { GetWindowTextA(find_res, title_buffer.as_ptr() as _, title_length + 1) };
        let title = unsafe {
            CString::from_vec_unchecked(title_buffer)
                .into_string()
                .map_err(|e| anyhow!("Error:Failed to convert window title : {e}"))
        }?;
        let visible = unsafe { IsWindowVisible(find_res) };
        println!("Window found: {title}, visible: {visible}");
        Ok(())
    }
}

fn enum_windows_titles() {
    unsafe {
        let mut window_titles: Vec<u8> = Vec::new();
        EnumWindows(
            Some(enum_windows_titles_callback),
            &mut window_titles as *mut _ as _,
        );
        for title in window_titles {
            println!("{}", title);
        }
    }
}

unsafe extern "system" fn enum_windows_titles_callback(
    window_handle: winapi::shared::windef::HWND,
    lParam: winapi::shared::minwindef::LPARAM,
) -> winapi::shared::minwindef::BOOL {
    let mut title_length = GetWindowTextLengthA(window_handle);
    if title_length > 0 {
        title_length += 1; // add space for the null terminator
        let mut title_buffer = vec![0u8; title_length as usize];
        GetWindowTextA(window_handle, title_buffer.as_mut_ptr() as _, title_length);
        let title = CString::from_vec_unchecked(title_buffer)
            .into_string()
            .unwrap();
        let is_visible = IsWindowVisible(window_handle);
        if is_visible > 0 {
            let window_titles = &mut *(lParam as *mut Vec<String>);
            window_titles.push(title);
        }
    }
    1 // continue enumeration
}

#[test]
fn test_find_and_focus_window() {
    // find_and_focus_window(&"Code.exe".to_string()).unwrap();
    enum_windows_titles();
}
