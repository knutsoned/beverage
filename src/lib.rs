use sickle_ui::prelude::Dropdown;

pub const DEFAULT_LOCALE: &str = "en-US";

pub mod construct;
pub mod framework;
pub mod input;
pub mod l10n;
pub mod layout;
pub mod remote;
pub mod router;
pub mod setup;
pub mod theme;
pub mod undo;
pub mod widget;

// plugins will want to have the domain objects available
pub mod prelude {
    pub use crate::framework::*;
    pub use crate::DEFAULT_LOCALE;
    pub use crate::get_selected_locale;
}

// map each language switcher dropdown option to a locale
pub fn get_selected_locale(locale_select: &Dropdown) -> String {
    match locale_select.value() {
        // this is the single source of truth for the locales the app is claiming to support on the backend

        // the options matched here must follow the order of the items in the dropdown or the wrong language will be chosen
        // (the default is option 0 so it doesn't need an explicit mapping)

        // FIXME there should be an ordered map of language codes to dropdown
        // ...and eventually proper locale management
        Some(1) => "fr-FR".to_string(),
        _ => DEFAULT_LOCALE.to_string(),
    }
}
