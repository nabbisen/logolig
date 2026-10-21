//! Tests for the window settings (RFC 025).

use super::*;

#[test]
fn app_id_is_the_reverse_dns_name_agreed_in_rfc_025() {
    assert_eq!(APP_ID, "io.github.nabbisen.Logolig");
}

#[cfg(target_os = "linux")]
#[test]
fn linux_settings_carry_the_application_id() {
    assert_eq!(settings().platform_specific.application_id, APP_ID);
}

/// Nothing but the application id differs from iced's defaults.
#[test]
fn everything_except_the_application_id_keeps_the_iced_defaults() {
    #[cfg_attr(not(target_os = "linux"), allow(unused_mut))]
    let mut ours = settings();
    #[cfg(target_os = "linux")]
    {
        ours.platform_specific.application_id = String::new();
    }
    assert_eq!(
        format!("{ours:?}"),
        format!("{:?}", window::Settings::default())
    );
}
