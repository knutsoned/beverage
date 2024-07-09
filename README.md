# Beverage

#### _Also available as a t-shirt._

---

## Overview

This is a framework for developing an editor for [Bevy](https://github.com/bevyengine/bevy).

This project is under heavy developmment. For now, the following demo showcases integration of
sickle_ui, bevy_fluent, leafwing-input-manager, and BRP (the unreleased bevy_remote protocol).

## Demo

In one terminal, cargo run --example server

![Initial Server Window State](./docs/bev1a.png)

In another, cargo run --example camera_control

![Initial Editor Window State](./docs/bev1b.png)

In the bottom left corner of the editor window, a label indicates the network connection status.
It is red when disconnected, yellow when a connection has been initiated but not resolved, and
green when a BRP query has returned with the ID of an entity with a Camera component.

Pressing the A or D keys will rotate the editor camera and then, once connected, begin updating the
transform of the server camera each frame that it changes. Clicking rotate scene while disconnected
will not initiate a connection, but if already connected, the server window will match the rotation
even if the user is no longer pressing the A or D key.

![Editor Remotely Controls Server Camera](./docs/bev2.png)

In addition, when connected, the F key will toggle an FPS counter in the server window.

![Editor Remotely Controls Server FPS Widget](./docs/bev3.png)

Selecting a setting in the right side of the menu bar will adjust the color theme settings of
sickle and switch the UI language using the asset-driven workflow of bevy_fluent.

![Editor Allows Language Selection](./docs/bev4.png)

Controls are mapped via leafwing-input-manager, except for the UI debug outlines which appear to be
hard-wired to the space bar.

## Design Questions

- How do we represent common entities over a BRP connection? The current impl just uses the u64
  EntityId bits serialized as a decimal integer string. This is not network safe.

## Roadmap

Three main things are on the horizon: managing assets for scenes and UI layouts and widgets;
providing an SDK and programming guide for editor plugin development; and bevy_mod_picking.

- [ ] Asset loading and viewing
- [ ] Serializing UI layouts
- [ ] Inflating serialized layouts
- [ ] Basic service framework
- [ ] Basic activity impl
- [ ] Focus and selection management
- [ ] Blender workflow integration
- [ ] UI widget editor
- [ ] Tree widget
- [ ] ECS property editor
- [ ] Scene editor

### Managing Assets

It should be possible now to integrate the asset-driven workflows for Blender and other 3D editors.
In addition, space_editor has a prefab system that can also serve as a reference. All file types,
least of all glTF, that have a supported viewer should have preview built in to the asset explorer.
Assets must have a way to be tagged for internal use i.e. atlases.

### Editor SDK

The basic internal architecture of the editor is designed to support various scenarios in game
development. It should provide services that allow a 3rd party plugin to build new functionality
that integrates with the core editor without altering the editor code.

Just make a Bevy app object, add the EditorPlugin after the DefaultPlugins, and then your own
plugins to get a fully customized editor.

### Picking

Being able to select and manipulate objects throughout the UI and within any scene editor views
in a consistent way is crucial to the above experience. Focus and selection are core parts of the
activity context that every editor plugin has access to as a regular resource. In addition some
care will be taken to ensure that accessibility is a priority, and there should always be a way
for a user with a single stick controller and 3rd party onscreen keyboard to utilize every provided
activity.

## Additional Background Noise

I started writing out a [rationale](rationale.md) and then decided to put my ongoing play-by-play
over [here](commentary.md).

## 🕊 Bevy Compatibility

| bevy   | beverage |
| ------ | -------- |
| 0.14.0 | 0.1.0    |

# License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

## Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
