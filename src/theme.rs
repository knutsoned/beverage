// this is all from the sickle simple_editor example.

use bevy::prelude::*;

use sickle_ui::prelude::*;

// theme handling widgets
#[derive(Component, Debug)]
pub struct ThemeSwitch;

#[derive(Component, Debug)]
pub struct ThemeContrastSelect;

pub fn handle_theme_data_update(
    theme_data: Res<ThemeData>,
    mut q_theme_switch: Query<&mut RadioGroup, With<ThemeSwitch>>,
    mut q_theme_contrast_select: Query<&mut Dropdown, With<ThemeContrastSelect>>
) {
    if theme_data.is_changed() {
        let Ok(mut theme_switch) = q_theme_switch.get_single_mut() else {
            return;
        };

        let Ok(mut theme_contrast_select) = q_theme_contrast_select.get_single_mut() else {
            return;
        };

        match theme_data.active_scheme {
            Scheme::Light(contrast) => {
                theme_switch.select(0);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                }
            }
            Scheme::Dark(contrast) => {
                theme_switch.select(1);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                }
            }
        };
    }
}

pub fn handle_theme_switch(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, (With<ThemeSwitch>, Changed<RadioGroup>)>,
    q_theme_contrast_select: Query<&Dropdown, With<ThemeContrastSelect>>
) {
    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

pub fn handle_theme_contrast_select(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, With<ThemeSwitch>>,
    q_theme_contrast_select: Query<&Dropdown, (With<ThemeContrastSelect>, Changed<Dropdown>)>
) {
    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

fn get_selected_scheme(
    theme_switch: &RadioGroup,
    theme_contrast_select: &Dropdown
) -> Option<Scheme> {
    let contrast = match theme_contrast_select.value() {
        Some(index) =>
            match index {
                0 => Contrast::Standard,
                1 => Contrast::Medium,
                2 => Contrast::High,
                _ => Contrast::Standard,
            }
        None => Contrast::Standard,
    };

    if let Some(index) = theme_switch.selected() {
        let scheme = match index {
            0 => Scheme::Light(contrast),
            1 => Scheme::Dark(contrast),
            _ => Scheme::Light(contrast),
        };

        Some(scheme)
    } else {
        None
    }
}
