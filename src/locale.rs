// Literally from the bevy_fluent UI example

// main difference is we don't store the locales in a hash. we just ask the AssetServer for the next one.

use bevy::{ asset::{ LoadState, LoadedFolder }, prelude::* };

use bevy_fluent::{ FluentPlugin, Locale, Localization, LocalizationBuilder };
use fluent_content::Content;
use unic_langid::LanguageIdentifier;

use sickle_ui::{ prelude::Dropdown, ui_commands::UpdateStatesExt };

use crate::prelude::*;

pub const DEFAULT_LOCALE: &str = "en-US";

pub struct EditorLocalePlugin;

impl Plugin for EditorLocalePlugin {
    fn build(&self, app: &mut App) {
        let default_li = DEFAULT_LOCALE.parse::<LanguageIdentifier>().expect(
            "Invalid default LanguageIdentifier"
        );

        app.add_plugins(FluentPlugin)
            // identify the current locale
            // (a different resource Localization uses this to look up actual string templates)
            .insert_resource(Locale::new(default_li))

            // init fluent l10n
            .add_systems(OnEnter(EditorState::Loading), setup)

            // handle selecting a new locale from the language switcher
            .add_systems(OnEnter(EditorState::SwitchLocale), switch_locale)

            // check to see if the AssetServer is done loading the locales folder
            .add_systems(Update, update.run_if(in_state(EditorState::Loading)))

            // check for a new selection from the language dropdown
            .add_systems(Update, handle_locale_select.run_if(in_state(EditorState::Running)));
    }
}

/// The locale service provides localization for the core UI and allows plugins to register their
/// own string template assets.
#[derive(Resource, Default, Debug)]
pub struct LocaleService {}

#[derive(Resource)]
pub struct LocaleFolder(pub Handle<LoadedFolder>);

#[derive(Component)]
pub struct LocaleRoot;

#[derive(Component, Debug)]
pub struct LocaleSelect;

// you can not add two pointers because two pointers can not be added
fn concat(prefix: &str, str: &str) -> String {
    prefix.to_string() + str
}

pub trait Translator {
    fn lbl(&self, str: &str) -> String;
    fn t(&self, string: String) -> String;
}

impl Translator for Localization {
    // we also add a t function to the Localization resource. this style is familiar to many developers who work with i18n and l10n.
    fn t(&self, string: String) -> String {
        match self.content(&string) {
            Some(string) => string,
            None => "XX".to_string(),
        }
    }

    // convenience function so the resource can be called with a short, arbitrarily-named method
    fn lbl(&self, str: &str) -> String {
        self.t(concat("lbl_", str))
    }
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    warn!("setup");
    // recursively load all the files in assets/locales
    let handle = asset_server.load_folder("locales");
    commands.insert_resource(LocaleFolder(handle));
}

pub fn switch_locale(
    mut commands: Commands,
    locale_folder: Res<LocaleFolder>,
    localization_builder: LocalizationBuilder
) {
    warn!("switch_locale");
    let localization = localization_builder.build(&locale_folder.0);
    commands.insert_resource(localization);

    // done switching, now rebuild UI
    commands.next_state(EditorState::Building);
}

pub fn update(
    mut commands: Commands,
    localization_builder: LocalizationBuilder,
    asset_server: Res<AssetServer>,
    locale_folder: Res<LocaleFolder>
) {
    // once locale_folder is loaded...
    if let Some(LoadState::Loaded) = asset_server.get_load_state(&locale_folder.0) {
        // create the main resource for i18n in UI template code
        let localization = localization_builder.build(&locale_folder.0);

        // we can't remove the LocaleFolder like in the example, otherwise we'd crash on locale switch

        // however this does mean hot reloading will work
        // NOTE: hot reloading is triggered lazily, so your changes won't currently show up until you
        //       trigger a reload by selecting a different language with the dropdown
        //commands.remove_resource::<LocaleFolder>();
        commands.insert_resource(localization);

        // done loading, switch to running
        commands.next_state(EditorState::Running);
    }
}

// the following describes a more dynamic approach to assigning the updated strings

/* TODO (from @koe on discord):
Afaict the best design for localization is to add a `LocalizedText` component to all entities with localized text.
This component contains `SmallVec<[(Option<LanguageIdentifier>, String); 1]>`, which is a vector of localization
templates for all the text sections on the entity, and settings like `sentence_case`. You need to store the
localization templates for the case where the user changes the game language at runtime and you need to replace all text.
It also buffers the templates for dynamic text so you can update the template without allocating (and then use the
main text section string to write the final localized text). On top of that, you need to store the language ID in case
you need it to replace certain fonts when swapping languages. In my code I have a `TextEditor` system parameter that
lets you write to a `Text` component directly without allocating, and localization integrates nicely with that.
*/

// handle language selection dropdown change
pub fn handle_locale_select(
    mut locale_resource: ResMut<Locale>,
    locale_select: Query<&Dropdown, (With<LocaleSelect>, Changed<Dropdown>)>,
    mut commands: Commands
) {
    // there can be only one (LocaleSelect Dropdown)
    let Ok(locale_select) = locale_select.get_single() else {
        return;
    };

    // convert dropdown selected index to a language ID string like "en-US"
    let langid = get_selected_locale(locale_select);

    // convert the string to an actual LanguageIdentifier
    if let Ok(langid) = langid.parse::<LanguageIdentifier>() {
        // compare the existing language ID string to the one we just selected
        if locale_resource.requested.to_string() != langid.to_string() {
            info!("Switching language to '{}'", langid);
            // replace the current requested language with the new one
            locale_resource.requested = langid.clone();

            // ...and add the default if it's not the default
            if langid.to_string() == *DEFAULT_LOCALE {
                locale_resource.default = None;
            } else {
                info!("-setting fallback locale '{}'", DEFAULT_LOCALE);
                locale_resource.default = Some(
                    DEFAULT_LOCALE.parse::<LanguageIdentifier>().unwrap()
                );
            }

            // trigger update of the UI text
            commands.next_state(EditorState::SwitchLocale);
            commands.next_state(Page::None);
        }
    }
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
