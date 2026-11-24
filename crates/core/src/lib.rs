//! RustMatte Core - Core image processing and model inference library.

/// Returns the operational status of the core library.
#[must_use]
pub fn status() -> &'static str {
    "rustmatte-core initialized"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_status() {
        assert_eq!(status(), "rustmatte-core initialized");
    }
}
