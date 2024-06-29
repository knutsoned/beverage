// Literally from the bevy_fluent UI example

use bevy::{ asset::{ LoadState, LoadedFolder }, prelude::* };

use bevy_fluent::*;
use fluent_content::Content;
use unic_langid::LanguageIdentifier;

use sickle_ui::prelude::Dropdown;

use crate::{ get_selected_locale, prelude::*, EditorState };

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // recursively load all the files in assets/locales
    let handle = asset_server.load_folder("locales");
    commands.insert_resource(LocaleFolder(handle));
}

pub fn update(
    mut commands: Commands,
    localization_builder: LocalizationBuilder,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<EditorState>>,
    locale_folder: Res<LocaleFolder>
) {
    // once locale_folder is loaded...
    if let Some(LoadState::Loaded) = asset_server.get_load_state(&locale_folder.0) {
        // create the main resource for i18n in UI template code
        let localization = localization_builder.build(&locale_folder.0);
        commands.remove_resource::<LocaleFolder>();
        commands.insert_resource(localization);

        // done loading, switch to running
        info!("Now run it...");
        next_state.set(EditorState::Running);
    }
}

// from sickle_ui basic_editor example
// handle language selection dropdown change
pub fn handle_locale_select(
    locale_data: Res<Locale>,
    q_locale_select: Query<&Dropdown, (With<LocaleSelect>, Changed<Dropdown>)>,
    mut commands: Commands
) {
    // there can be only one (LocaleSelect Dropdown)
    let Ok(locale_select) = q_locale_select.get_single() else {
        return;
    };

    // convert dropdown selected index to a language ID string like "en-US"
    let locale = get_selected_locale(locale_select);

    // convert the string to an actual LanguageIdentifier
    if let Ok(locale) = locale.parse::<LanguageIdentifier>() {
        // compare the existing language ID string to the one we just selected
        if locale_data.requested.to_string() != locale.to_string() {
            // prepare a new Locale resource
            let mut locale = Locale::new(locale);

            // ...and add the default if it's not the default
            if locale.requested.to_string() != *DEFAULT_LOCALE {
                locale = locale.with_default(DEFAULT_LOCALE.parse::<LanguageIdentifier>().unwrap());
            }

            // replace the current Locale with the new resource
            info!("Switching language to '{}'", locale.requested.to_string());
            commands.insert_resource(locale);
        }
    }
}

// new code
fn concat(prefix: &str, str: &str) -> String {
    prefix.to_string() + str
}

// convenience function so the resource can be called with a short, arbitrarily-named method
impl Translator for Localization {
    fn lbl(&self, str: &str) -> String {
        self.t(concat("lbl_", str))
    }

    fn t(&self, string: String) -> String {
        match self.content(&string) {
            Some(string) => string,
            None => "XX".to_string(),
        }
    }
}

#[derive(Resource)]
pub struct LocaleFolder(Handle<LoadedFolder>);
