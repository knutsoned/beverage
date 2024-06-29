use bevy::state::state::States;

use sickle_ui::prelude::Dropdown;

pub const DEFAULT_LOCALE: &str = "en-US";

pub mod framework;
pub mod l10n;
pub mod layout;
pub mod setup;
pub mod theme;

// plugins will want to have the domain objects available
pub mod prelude {
    pub use crate::framework::*;
    pub use crate::DEFAULT_LOCALE;
}

// map each language switcher dropdown option to a locale
fn get_selected_locale(locale_select: &Dropdown) -> String {
    match locale_select.value() {
        // this is the single source of truth for the locales the app is claiming to support on the backend

        // the options matched here must follow the order of the items in the dropdown or the wrong language will be chosen
        // (the default is option 0 so it doesn't need an explicit mapping)

        // FIXME there should be an ordered map of language codes to dropdown labels
        Some(1) => "fr-FR".to_string(),
        _ => DEFAULT_LOCALE.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum EditorState {
    #[default]
    Loading,
    Running,
}
