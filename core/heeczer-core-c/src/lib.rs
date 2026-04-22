//! C ABI surface for `heeczer-core`. Used by Go (cgo) and Java (FFM) bindings.
//!
//! Memory ownership: any `*mut c_char` returned by this crate MUST be freed via
//! [`heeczer_free_string`]. Inputs are borrowed; the caller retains ownership.
//!
//! All functions are `extern "C"` and panic-safe. A panic inside Rust is
//! caught and converted to an error string in the result envelope.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

use heeczer_core::{score, Event, ScoringProfile, TierSet};

/// Free a string previously returned by this library. Passing a null pointer is a no-op.
#[no_mangle]
pub unsafe extern "C" fn heeczer_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = unsafe { CString::from_raw(s) };
}

/// Score an event. All arguments are NUL-terminated UTF-8 JSON strings.
///
/// `profile_json` and `tiers_json` may be NULL to use the embedded defaults.
/// `tier_override` may be NULL.
///
/// Returns a NUL-terminated JSON envelope:
///   `{"ok": true, "result": <ScoreResult>}` on success
///   `{"ok": false, "error": "..."}` on failure
///
/// The returned pointer must be released via [`heeczer_free_string`].
#[no_mangle]
pub unsafe extern "C" fn heeczer_score_json(
    event_json: *const c_char,
    profile_json: *const c_char,
    tiers_json: *const c_char,
    tier_override: *const c_char,
) -> *mut c_char {
    let response = catch_unwind(AssertUnwindSafe(|| {
        let event_s = unsafe { cstr(event_json) }.ok_or("event_json must be non-null")?;
        // Parse once and gate on the canonical schema before materialising the
        // typed Event. Every non-Rust SDK funnels through this entry point and
        // must see the same strict-mode rejection the CLI gives (PRD §13).
        let event_value: serde_json::Value =
            serde_json::from_str(event_s).map_err(|e| e.to_string())?;
        heeczer_core::schema::EventValidator::new_v1()
            .validate(&event_value, heeczer_core::schema::Mode::Strict)
            .map_err(|e| e.to_string())?;
        let event: Event = serde_json::from_value(event_value).map_err(|e| e.to_string())?;

        let profile = if profile_json.is_null() {
            ScoringProfile::default_v1()
        } else {
            let s = unsafe { cstr(profile_json) }.unwrap_or("");
            serde_json::from_str(s).map_err(|e| e.to_string())?
        };

        let tiers = if tiers_json.is_null() {
            TierSet::default_v1()
        } else {
            let s = unsafe { cstr(tiers_json) }.unwrap_or("");
            serde_json::from_str(s).map_err(|e| e.to_string())?
        };

        let tier_override_s = if tier_override.is_null() {
            None
        } else {
            unsafe { cstr(tier_override) }
        };

        let result = score(&event, &profile, &tiers, tier_override_s).map_err(|e| e.to_string())?;
        let body = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        Ok::<String, String>(format!(r#"{{"ok":true,"result":{body}}}"#))
    }));

    let s = match response {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => format!(r#"{{"ok":false,"error":{}}}"#, json_string(&e)),
        Err(_) => r#"{"ok":false,"error":"panic in heeczer_score_json"}"#.to_string(),
    };
    CString::new(s)
        .unwrap_or_else(|_| CString::new("{\"ok\":false,\"error\":\"nul in result\"}").unwrap())
        .into_raw()
}

/// Return the embedded scoring/spec versions as a JSON string. Caller frees.
#[no_mangle]
pub extern "C" fn heeczer_versions_json() -> *mut c_char {
    let s = format!(
        r#"{{"scoring_version":"{}","spec_version":"{}"}}"#,
        heeczer_core::SCORING_VERSION,
        heeczer_core::SPEC_VERSION,
    );
    CString::new(s).unwrap().into_raw()
}

unsafe fn cstr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }.to_str().ok()
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn score_via_c_abi_round_trips() {
        let body = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../schema/fixtures/events/valid/01-prd-canonical.json"),
        )
        .unwrap();
        let cs_event = cs(&body);
        let out = unsafe {
            heeczer_score_json(
                cs_event.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        assert!(!out.is_null());
        let view = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_owned();
        assert!(view.contains("\"ok\":true"), "got: {view}");
        assert!(view.contains("\"scoring_version\":\"1.0.0\""));
        unsafe { heeczer_free_string(out) };
    }

    #[test]
    fn score_with_invalid_json_returns_error_envelope() {
        let bad = cs("{ not json }");
        let out = unsafe {
            heeczer_score_json(
                bad.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        let view = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_owned();
        assert!(view.contains("\"ok\":false"));
        unsafe { heeczer_free_string(out) };
    }

    #[test]
    fn null_pointer_is_handled_gracefully() {
        let out = unsafe {
            heeczer_score_json(
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        let view = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_owned();
        assert!(view.contains("\"ok\":false"));
        unsafe { heeczer_free_string(out) };
    }
}
