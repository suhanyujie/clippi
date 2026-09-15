//! Whether macOS is in secure keyboard entry mode.
//!
//! An application turns this on while a password field has focus (Chromium
//! browsers do it for their own password fields). While it is on the system
//! refuses to hand the keyboard to anyone else: `NSRunningApplication.activate`
//! returns false and the panel comes up behind the browser with no way to type
//! into it. That is the operating system protecting a password field, not
//! something to work around — but it is worth detecting, so the panel can say
//! what happened instead of flashing once and vanishing.

#[cfg(target_os = "macos")]
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn IsSecureEventInputEnabled() -> bool;
}

/// True while some application holds secure keyboard entry.
#[cfg(target_os = "macos")]
pub fn is_enabled() -> bool {
    // SAFETY: a parameterless Carbon predicate with no side effects.
    unsafe { IsSecureEventInputEnabled() }
}

#[cfg(not(target_os = "macos"))]
pub fn is_enabled() -> bool {
    false
}
