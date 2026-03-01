use std::process::Command;

/// Check if a text input field is currently focused using macOS Accessibility API
pub fn has_focused_text_input() -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;

        extern "C" {
            fn AXUIElementCreateSystemWide() -> *mut c_void;
            fn AXUIElementCopyAttributeValue(
                element: *mut c_void,
                attribute: *mut c_void,
                value: *mut *mut c_void,
            ) -> i32;
            fn CFRelease(cf: *const c_void);
        }

        use objc::{class, msg_send, sel, sel_impl};
        use objc::runtime::Object;

        unsafe {
            let system_wide = AXUIElementCreateSystemWide();
            if system_wide.is_null() {
                return false;
            }

            // Get AXFocusedUIElement
            let attr_name: *mut Object =
                msg_send![class!(NSString), stringWithUTF8String: b"AXFocusedUIElement\0".as_ptr()];
            let mut focused_element: *mut c_void = std::ptr::null_mut();
            let result = AXUIElementCopyAttributeValue(
                system_wide,
                attr_name as *mut c_void,
                &mut focused_element,
            );

            CFRelease(system_wide as *const c_void);

            // kAXErrorSuccess = 0
            if result != 0 || focused_element.is_null() {
                return false;
            }

            // Get AXRole of focused element
            let role_attr: *mut Object =
                msg_send![class!(NSString), stringWithUTF8String: b"AXRole\0".as_ptr()];
            let mut role_value: *mut c_void = std::ptr::null_mut();
            let role_result = AXUIElementCopyAttributeValue(
                focused_element,
                role_attr as *mut c_void,
                &mut role_value,
            );

            CFRelease(focused_element as *const c_void);

            if role_result != 0 || role_value.is_null() {
                return false;
            }

            // Convert CFString role to Rust string
            let role_nsstring = role_value as *mut Object;
            let utf8_ptr: *const u8 = msg_send![role_nsstring, UTF8String];
            let role_str = if !utf8_ptr.is_null() {
                std::ffi::CStr::from_ptr(utf8_ptr as *const i8)
                    .to_str()
                    .unwrap_or("")
            } else {
                ""
            };

            let is_text_input = matches!(
                role_str,
                "AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField" | "AXWebArea"
            );

            CFRelease(role_value as *const c_void);

            is_text_input
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS, assume text field is focused (fallback to paste behavior)
        true
    }
}

/// Get the name of the frontmost (active) application on macOS
pub fn get_frontmost_app() -> Result<String, anyhow::Error> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to get name of first application process whose frontmost is true"#)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("Unknown".to_string())
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok("Unknown".to_string())
    }
}

/// Check if microphone permission is granted using AVFoundation
pub fn check_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc::{msg_send, sel, sel_impl, class};
        use objc::runtime::Object;
        unsafe {
            // AVMediaTypeAudio = "soun"
            let media_type: *mut Object = msg_send![class!(NSString), stringWithUTF8String: b"soun\0".as_ptr()];
            // AVAuthorizationStatus: 0=notDetermined, 1=restricted, 2=denied, 3=authorized
            let status: i64 = msg_send![class!(AVCaptureDevice), authorizationStatusForMediaType: media_type];
            status == 3 // authorized
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Check if accessibility permission is granted using AXIsProcessTrusted
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        unsafe { AXIsProcessTrusted() }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Request microphone permission — triggers the macOS dialog
pub fn request_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc::{msg_send, sel, sel_impl, class};
        use objc::runtime::Object;
        use std::sync::{Arc, Mutex};

        unsafe {
            let media_type: *mut Object = msg_send![class!(NSString), stringWithUTF8String: b"soun\0".as_ptr()];

            // Check current status first
            let status: i64 = msg_send![class!(AVCaptureDevice), authorizationStatusForMediaType: media_type];

            if status == 0 {
                // Not determined — request access (this triggers the system dialog)
                let result = Arc::new(Mutex::new(false));
                let result_clone = result.clone();

                // Use a block to request access
                let block = block::ConcreteBlock::new(move |granted: bool| {
                    if let Ok(mut r) = result_clone.lock() {
                        *r = granted;
                    }
                });
                let block = block.copy();

                let _: () = msg_send![class!(AVCaptureDevice), requestAccessForMediaType: media_type completionHandler: &*block];

                // Give the dialog time to appear
                std::thread::sleep(std::time::Duration::from_millis(500));
                return *result.lock().unwrap_or_else(|e| e.into_inner());
            }

            status == 3 // already authorized
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Request accessibility permission — opens the system prompt
pub fn request_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
        }
        use objc::{msg_send, sel, sel_impl, class};
        use objc::runtime::Object;

        unsafe {
            // Create options dict with kAXTrustedCheckOptionPrompt = true
            let key: *mut Object = msg_send![class!(NSString), stringWithUTF8String: b"AXTrustedCheckOptionPrompt\0".as_ptr()];
            let value: *mut Object = msg_send![class!(NSNumber), numberWithBool: true];
            let dict: *mut Object = msg_send![class!(NSDictionary), dictionaryWithObject: value forKey: key];
            AXIsProcessTrustedWithOptions(dict as *const std::ffi::c_void)
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}
