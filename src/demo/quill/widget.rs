use bevy::{
    color::palettes,
    ecs::storage::SparseSet,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d,
            TextureDescriptor,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
    },
    ui::widget::UiImageSize,
    utils::HashMap,
};

use bevy_fluent::Localization;

use leafwing_input_manager::{ action_state::ActionState, input_map::InputMap, InputManagerBundle };

use sickle_ui::{ prelude::*, widgets::inputs::slider::SliderAxis };

use crate::{ framework::*, input::* };

pub struct QuillDemoPlugin;

impl Plugin for QuillDemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveQuillDemos>()
            .configure_sets(Update, SpawnQuillDemoUpdate.after(WidgetLibraryUpdate))
            .add_plugins(ComponentThemePlugin::<QuillDemoControl>::default())
            .add_systems(
                PreUpdate,
                (
                    spawn_quill_demo,
                    cleanup_despawned_quill_demos,
                    set_quill_demo_cam_viewport,
                    update_quill_demos,
                )
                    .in_set(SpawnQuillDemoPreUpdate)
                    .run_if(in_state(EditorState::Running))
            )
            .add_systems(
                Update,
                (process_quill_demo_controls, update_quill_demo_controls)
                    .chain()
                    .in_set(SpawnQuillDemoUpdate)
                    .run_if(in_state(EditorState::Running))
            )
            .add_systems(OnExit(Page::QuillDemo), despawn_active_camera);
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnQuillDemoPreUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnQuillDemoUpdate;

type QuillDemoResources<'a> = (
    ResMut<'a, ActiveQuillDemos>,
    ResMut<'a, Assets<Image>>,
    ResMut<'a, Assets<Mesh>>,
    ResMut<'a, Assets<StandardMaterial>>,
);

fn spawn_quill_demo((q_spawn_quill_demo, mut quill_demo_resources, l10n, mut commands): (
    Query<Entity, Added<SpawnQuillDemo>>,
    QuillDemoResources,
    Res<Localization>,
    Commands,
)) {
    for container in &q_spawn_quill_demo {
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        layout(container, size, &l10n, &mut quill_demo_resources, &mut commands);
    }
}

// TODO router lifecycle
/*
pub fn spawn_camera_tree_view(
    q_added_camera_control: Query<&CameraControl, Added<CameraControl>>,
    q_tree_view_panel: Query<Entity, With<TreeViewPanel>>,

    mut commands: Commands
) {
    if let Some(camera_control) = (&q_added_camera_control).into_iter().next() {
        let Ok(container) = q_tree_view_panel.get_single() else {
            return;
        };

        commands.entity(container).despawn_descendants();
        commands.ui_builder(container).tree_for(camera_control.asset_root());
    }
}
*/

// try to clean up camera and light when we navigate away
fn despawn_active_camera(
    remote_cameras: Query<Entity, With<RemoteCamera>>,
    mut commands: Commands
) {
    warn!("despawn_active_camera");
    for entity in remote_cameras.iter() {
        info!("-despawning {}", entity);
        commands.entity(entity).despawn_recursive();
    }
}

pub fn despawn_camera_tree_view(
    q_hierarchy_panel: Query<Entity, With<TreeViewPanel>>,
    q_removed_scene_view: RemovedComponents<QuillDemo>,
    mut commands: Commands
) {
    let Ok(container) = q_hierarchy_panel.get_single() else {
        return;
    };

    if !q_removed_scene_view.is_empty() {
        commands.entity(container).despawn_descendants();
    }
}

// layout
fn layout(
    container: Entity,
    size: Extent3d,
    l10n: &Res<Localization>,
    (active_quill_demos, images, meshes, materials): &mut QuillDemoResources,
    commands: &mut Commands
) {
    warn!("layout");
    /*
    match editor {
        EditorState::Loading => info!("Loading"),
        EditorState::SwitchLocale => info!("SwitchLocale"),
        EditorState::Building => info!("Building"),
        EditorState::Running => info!("Running"),
    }
    match connection {
        RemoteConnectionState::Disconnected => info!("Disconnected"),
        RemoteConnectionState::Connecting => info!("Connecting"),
        RemoteConnectionState::Checking => info!("Checking"),
        RemoteConnectionState::Connected => info!("Connected"),
    }
    */

    // for the Camera Control demo, KeyA and KeyD rotate around Y in opposite directions
    // the F key toggles the remote FPS counter
    let mut input_map = InputMap::new([
        (InputAction::CameraRotateYDecrease, KeyCode::KeyA),
        (InputAction::CameraRotateYIncrease, KeyCode::KeyD),
        (InputAction::ToggleRemoteFpsCounter, KeyCode::KeyF),
    ]);

    // we will also accept West and East
    input_map.insert(InputAction::CameraRotateYDecrease, GamepadButtonType::West);
    input_map.insert(InputAction::CameraRotateYIncrease, GamepadButtonType::East);

    // sample scene objects
    // circular base
    let scene_ground = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Circle::new(4.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_rotation(
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
            ),
            ..default()
        })
        .id();

    // cube
    let scene_cube = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .id();

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING |
            TextureUsages::COPY_DST |
            TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);
    let image_handle = images.add(image);

    let scene_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    clear_color: ClearColorConfig::Custom(palettes::css::DARK_GRAY.into()),
                    order: 0,
                    target: image_handle.clone().into(),
                    ..default()
                },
                // sickle example: Transform::from_xyz(0.0, 2.0, -3.0).looking_at(
                transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            FogSettings {
                color: Color::srgb(0.25, 0.25, 0.25),
                falloff: FogFalloff::Linear {
                    start: 7.0,
                    end: 12.0,
                },
                ..default()
            },
            RemoteCamera,
        ))
        .id();

    let transform = Transform::from_xyz(0.0, 10.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y);
    let scene_light = commands
        .spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    color: Color::srgb(1.0, 0.953, 0.886),
                    shadows_enabled: true,
                    ..default()
                },
                transform,
                ..default()
            },
        ))
        .id();

    // spawn the main CameraControl
    commands
        .entity(container)
        .insert((
            QuillDemo {
                camera: scene_camera,
                cube: scene_cube,
                ground: scene_ground,
                light: scene_light,
            },
            QuillDemoSettings::default(),
            InputManagerBundle::with_map(input_map),
            UiImage::new(image_handle),
            UiImageSize::default(),
        ))
        .remove::<SpawnQuillDemo>();

    commands.ui_builder(container).row(|scene_controls| {
        let radio_group_theme = PseudoTheme::build(None, |style_builder| {
            style_builder.flex_wrap(FlexWrap::NoWrap);
        });

        scene_controls.insert((
            QuillDemoControl {
                quill_demo: container,
            },
            Theme::<RadioGroup>::new(vec![radio_group_theme]),
        ));

        scene_controls.checkbox(l10n.lbl("RotateScene"), false).insert(SceneRotationControl {
            quill_demo_control: container,
        });
        scene_controls
            .slider(
                SliderConfig::new(
                    l10n.lbl("RotationSpeed"),
                    -1.0,
                    1.0,
                    0.1,
                    true,
                    SliderAxis::Horizontal
                )
            )
            .insert(SceneRotationSpeedControl {
                quill_demo_control: container,
            })
            .style()
            .min_width(Val::Px(250.0));
        scene_controls
            .row(|row| {
                row.radio_group(
                    vec![l10n.lbl("Natural"), l10n.lbl("Dim"), l10n.lbl("Night")],
                    1,
                    true
                ).insert(SceneLightControl {
                    quill_demo_control: container,
                });
            })
            .style()
            .min_width(Val::Px(150.0));
    });

    active_quill_demos.quill_demo.insert(container, QuillDemo {
        camera: scene_camera,
        cube: scene_cube,
        ground: scene_ground,
        light: scene_light,
    });
}

fn cleanup_despawned_quill_demos(
    mut q_removed_quill_demos: RemovedComponents<QuillDemo>,
    mut active_quill_demos: ResMut<ActiveQuillDemos>,
    mut commands: Commands
) {
    for entity in q_removed_quill_demos.read() {
        let Some(data) = active_quill_demos.quill_demo.remove(&entity) else {
            error!("Tried to clean up untracked scene view {:?}", entity);
            continue;
        };

        commands.entity(data.camera).despawn_recursive();
        commands.entity(data.light).despawn_recursive();
    }
}

fn set_quill_demo_cam_viewport(
    q_quill_demos: Query<(&QuillDemo, &Node), Changed<GlobalTransform>>,
    mut images: ResMut<Assets<Image>>,
    mut q_camera: Query<&mut Camera>
) {
    for (quill_demo, node) in &q_quill_demos {
        let Ok(mut camera) = q_camera.get_mut(quill_demo.camera()) else {
            continue;
        };

        let size = node.size();

        if size.x == 0.0 || size.y == 0.0 {
            camera.is_active = false;
            continue;
        }

        camera.is_active = true;

        if let RenderTarget::Image(render_texture) = camera.target.clone() {
            let Some(texture) = images.get_mut(&render_texture) else {
                continue;
            };

            let size = Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                ..default()
            };

            texture.resize(size);
        }
    }
}

fn update_quill_demo_controls(
    q_quill_demo_settings: Query<&QuillDemoSettings, Changed<QuillDemoSettings>>,
    mut q_rotation_controls: Query<(&mut Checkbox, &SceneRotationControl)>,
    mut q_rotation_speed_controls: Query<(&mut Slider, &SceneRotationSpeedControl)>,
    mut q_light_controls: Query<(&mut RadioGroup, &SceneLightControl)>
) {
    for (mut checkbox, control) in &mut q_rotation_controls {
        let Ok(settings) = q_quill_demo_settings.get(control.quill_demo_control) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            checkbox.checked = settings.do_rotate;
        }
    }

    for (mut slider, control) in &mut q_rotation_speed_controls {
        let Ok(settings) = q_quill_demo_settings.get(control.quill_demo_control) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            slider.set_value(settings.rotation_speed);
        }
    }

    for (mut radio_group, control) in &mut q_light_controls {
        let Ok(settings) = q_quill_demo_settings.get(control.quill_demo_control) else {
            continue;
        };

        if radio_group.selected != settings.light.into() {
            radio_group.selected = settings.light.into();
        }
    }
}

fn process_quill_demo_controls(
    mut q_quill_demo_settings: Query<&mut QuillDemoSettings>,
    q_rotation_controls: Query<(&Checkbox, &SceneRotationControl), Changed<Checkbox>>,
    q_rotation_speed_controls: Query<(&Slider, &SceneRotationSpeedControl), Changed<Slider>>,
    q_light_controls: Query<(&RadioGroup, &SceneLightControl), Changed<RadioGroup>>
) {
    for (checkbox, control) in &q_rotation_controls {
        let Ok(mut settings) = q_quill_demo_settings.get_mut(control.quill_demo_control) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            settings.do_rotate = checkbox.checked;
        }
    }

    for (slider, control) in &q_rotation_speed_controls {
        let Ok(mut settings) = q_quill_demo_settings.get_mut(control.quill_demo_control) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            settings.rotation_speed = slider.value();
        }
    }

    for (radio_group, control) in &q_light_controls {
        let Ok(mut settings) = q_quill_demo_settings.get_mut(control.quill_demo_control) else {
            continue;
        };

        if radio_group.selected != settings.light.into() {
            let Some(light) = radio_group.selected else {
                continue;
            };
            settings.light = light;
        }
    }
}

fn update_quill_demos(
    time: Res<Time>,
    q_action: Query<(Entity, &ActionState<InputAction>), With<QuillDemo>>,
    q_quill_demos: Query<(Entity, &QuillDemo, Ref<QuillDemoSettings>)>,
    mut ambient_light: ResMut<AmbientLight>,
    mut q_directional_light: Query<&mut DirectionalLight>,
    mut q_fog_settings: Query<&mut FogSettings>,
    mut q_transform: Query<&mut Transform>
) {
    // pause automatic rotation if the user is controlling
    let mut manual_rotation = SparseSet::<Entity, i32>::new();
    for (entity, action_state) in &q_action {
        let left = action_state.pressed(&InputAction::CameraRotateYIncrease);
        let right = action_state.pressed(&InputAction::CameraRotateYDecrease);
        if right {
            manual_rotation.insert(entity, 1);
        } else if left {
            manual_rotation.insert(entity, -1);
        } else {
            manual_rotation.insert(entity, 0);
        }
    }

    for (entity, quill_demo, settings) in &q_quill_demos {
        let Ok(mut transform) = q_transform.get_mut(quill_demo.camera()) else {
            continue;
        };
        if settings.rotation_speed != 0.0 {
            let manual_rotation = manual_rotation.get(entity);
            match manual_rotation {
                Some(0) => {
                    // auto-rotation based on the Rotate Scene checkbox
                    if settings.do_rotate {
                        transform.rotate_around(
                            Vec3::ZERO,
                            Quat::from_euler(
                                EulerRot::default(),
                                -time.delta_seconds() * settings.rotation_speed,
                                0.0,
                                0.0
                            )
                        );
                    }
                }
                Some(direction) => {
                    let direction = *direction as f32;

                    // manual rotation from inputs
                    transform.rotate_around(
                        Vec3::ZERO,
                        Quat::from_euler(
                            EulerRot::default(),
                            -time.delta_seconds() * settings.rotation_speed * direction,
                            0.0,
                            0.0
                        )
                    );
                }
                _ => {}
            }
        }

        if settings.is_changed() {
            let Ok(mut light) = q_directional_light.get_mut(quill_demo.light()) else {
                continue;
            };
            let Ok(mut fog) = q_fog_settings.get_mut(quill_demo.camera()) else {
                continue;
            };

            match settings.light {
                0 => {
                    light.color = Color::srgb(1.0, 0.953, 0.886);
                    light.illuminance = 13500.0;
                    ambient_light.brightness = 500.0;
                    fog.falloff = FogFalloff::Linear {
                        start: 7.0,
                        end: 12.0,
                    };
                }
                1 => {
                    light.color = Color::srgb(0.78, 0.76, 0.745);
                    light.illuminance = 9000.0;
                    ambient_light.brightness = 300.0;
                    fog.falloff = FogFalloff::Linear {
                        start: 6.0,
                        end: 15.0,
                    };
                }
                2 => {
                    light.color = Color::srgb(0.73, 0.9, 0.95); // Color::rgb(0.53, 0.8, 0.92);
                    light.illuminance = 300.0;
                    ambient_light.brightness = 5.0;
                    fog.falloff = FogFalloff::Linear {
                        start: 5.0,
                        end: 20.0,
                    };
                }
                _ => (),
            };
        }
    }
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct ActiveQuillDemos {
    quill_demo: HashMap<Entity, QuillDemo>,
}

impl Default for ActiveQuillDemos {
    fn default() -> Self {
        Self {
            quill_demo: HashMap::new(),
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, UiContext)]
#[reflect(Component)]
struct QuillDemoControl {
    quill_demo: Entity,
}

impl Default for QuillDemoControl {
    fn default() -> Self {
        Self {
            quill_demo: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for QuillDemoControl {
    fn default_theme() -> Option<Theme<QuillDemoControl>> {
        QuillDemoControl::theme().into()
    }
}

impl QuillDemoControl {
    pub fn theme() -> Theme<QuillDemoControl> {
        let base_theme = PseudoTheme::deferred(None, QuillDemoControl::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .justify_self(JustifySelf::Start)
            .height(Val::Px(theme_spacing.areas.small))
            .position_type(PositionType::Absolute)
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneRotationControl {
    quill_demo_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneRotationSpeedControl {
    quill_demo_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneLightControl {
    quill_demo_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
struct SpawnQuillDemo;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct QuillDemo {
    camera: Entity,
    cube: Entity,
    ground: Entity,
    light: Entity,
}

impl Default for QuillDemo {
    fn default() -> Self {
        Self {
            camera: Entity::PLACEHOLDER,
            cube: Entity::PLACEHOLDER,
            ground: Entity::PLACEHOLDER,
            light: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct QuillDemoSettings {
    do_rotate: bool,
    rotation_speed: f32,
    light: usize,
}

impl Default for QuillDemoSettings {
    fn default() -> Self {
        Self {
            do_rotate: true,
            rotation_speed: 0.1,
            light: 0,
        }
    }
}

impl QuillDemo {
    pub fn camera(&self) -> Entity {
        self.camera
    }

    pub fn light(&self) -> Entity {
        self.light
    }
}

pub trait UiQuillDemoExt {
    fn quill_demo(&mut self) -> UiBuilder<Entity>;
}

impl UiQuillDemoExt for UiBuilder<'_, Entity> {
    fn quill_demo(&mut self) -> UiBuilder<Entity> {
        let column = self
            //.column(|_| {})
            .insert((Name::new("Camera Control"), SpawnQuillDemo))
            .style() // Needed until UiImage stops depending on background color
            .background_color(Color::WHITE)
            .width(Val::Percent(100.0))
            .id();

        self.commands().ui_builder(column)
    }
}
