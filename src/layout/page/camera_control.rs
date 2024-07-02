use bevy::{
    color::palettes,
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

use sickle_ui::{ prelude::*, widgets::inputs::slider::SliderAxis };

use crate::framework::*;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveCameraControls>()
            .configure_sets(Update, SpawnCameraControlUpdate.after(WidgetLibraryUpdate))
            .add_plugins(ComponentThemePlugin::<CameraControls>::default())
            .add_systems(
                PreUpdate,
                (
                    spawn_camera_control,
                    cleanup_despawned_camera_controls,
                    set_camera_control_cam_viewport,
                    update_camera_controls,
                )
                    .in_set(SpawnCameraControlPreUpdate)
                    .run_if(in_state(EditorState::Running))
            )
            .add_systems(
                Update,
                (process_camera_control_controls, update_camera_control_controls)
                    .chain()
                    .in_set(SpawnCameraControlUpdate)
                    .run_if(in_state(EditorState::Running))
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnCameraControlPreUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnCameraControlUpdate;

fn spawn_camera_control(
    q_spawn_camera_control: Query<Entity, Added<SpawnCameraControl>>,
    l10n: Res<Localization>,
    mut active_camera_controls: ResMut<ActiveCameraControls>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands
) {
    for container in &q_spawn_camera_control {
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        layout(container, size, &l10n, &mut active_camera_controls, &mut images, &mut commands);
    }
}

fn layout(
    container: Entity,
    size: Extent3d,
    l10n: &Res<Localization>,
    active_camera_controls: &mut ResMut<ActiveCameraControls>,
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands
) {
    warn!("layout");
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
                transform: Transform::from_xyz(0.0, 2.0, -3.0).looking_at(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::Y
                ),
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
        ))
        .id();

    let transform = Transform::from_xyz(0.0, 10.0, 3.0).looking_at(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::Y
    );
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

    let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
    transform.scale = Vec3::splat(1.0);

    commands
        .entity(container)
        .insert((
            CameraControl {
                camera: scene_camera,
                light: scene_light,
            },
            CameraControlSettings::default(),
            UiImage::new(image_handle),
            UiImageSize::default(),
        ))
        .remove::<SpawnCameraControl>();

    commands.ui_builder(container).row(|scene_controls| {
        let radio_group_theme = PseudoTheme::build(None, |style_builder| {
            style_builder.flex_wrap(FlexWrap::NoWrap);
        });

        scene_controls.insert((
            CameraControls {
                camera_control: container,
            },
            Theme::<RadioGroup>::new(vec![radio_group_theme]),
        ));

        scene_controls.checkbox(l10n.lbl("RotateScene"), false).insert(SceneRotationControl {
            camera_control: container,
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
                camera_control: container,
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
                    camera_control: container,
                });
            })
            .style()
            .min_width(Val::Px(150.0));
    });

    active_camera_controls.camera_controls.insert(container, CameraControl {
        camera: scene_camera,
        light: scene_light,
    });
}

fn cleanup_despawned_camera_controls(
    mut q_removed_camera_controls: RemovedComponents<CameraControl>,
    mut active_camera_controls: ResMut<ActiveCameraControls>,
    mut commands: Commands
) {
    for entity in q_removed_camera_controls.read() {
        let Some(data) = active_camera_controls.camera_controls.remove(&entity) else {
            error!("Tried to clean up untracked scene view {:?}", entity);
            continue;
        };

        commands.entity(data.camera).despawn_recursive();
        commands.entity(data.light).despawn_recursive();
    }
}

fn set_camera_control_cam_viewport(
    q_camera_controls: Query<(&CameraControl, &Node), Changed<GlobalTransform>>,
    mut images: ResMut<Assets<Image>>,
    mut q_camera: Query<&mut Camera>
) {
    for (camera_control, node) in &q_camera_controls {
        let Ok(mut camera) = q_camera.get_mut(camera_control.camera()) else {
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

fn update_camera_control_controls(
    q_camera_control_settings: Query<&CameraControlSettings, Changed<CameraControlSettings>>,
    mut q_rotation_controls: Query<(&mut Checkbox, &SceneRotationControl)>,
    mut q_rotation_speed_controls: Query<(&mut Slider, &SceneRotationSpeedControl)>,
    mut q_light_controls: Query<(&mut RadioGroup, &SceneLightControl)>
) {
    for (mut checkbox, control) in &mut q_rotation_controls {
        let Ok(settings) = q_camera_control_settings.get(control.camera_control) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            checkbox.checked = settings.do_rotate;
        }
    }

    for (mut slider, control) in &mut q_rotation_speed_controls {
        let Ok(settings) = q_camera_control_settings.get(control.camera_control) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            slider.set_value(settings.rotation_speed);
        }
    }

    for (mut radio_group, control) in &mut q_light_controls {
        let Ok(settings) = q_camera_control_settings.get(control.camera_control) else {
            continue;
        };

        if radio_group.selected != settings.light.into() {
            radio_group.selected = settings.light.into();
        }
    }
}

fn process_camera_control_controls(
    mut q_camera_control_settings: Query<&mut CameraControlSettings>,
    q_rotation_controls: Query<(&Checkbox, &SceneRotationControl), Changed<Checkbox>>,
    q_rotation_speed_controls: Query<(&Slider, &SceneRotationSpeedControl), Changed<Slider>>,
    q_light_controls: Query<(&RadioGroup, &SceneLightControl), Changed<RadioGroup>>
) {
    for (checkbox, control) in &q_rotation_controls {
        let Ok(mut settings) = q_camera_control_settings.get_mut(control.camera_control) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            settings.do_rotate = checkbox.checked;
        }
    }

    for (slider, control) in &q_rotation_speed_controls {
        let Ok(mut settings) = q_camera_control_settings.get_mut(control.camera_control) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            settings.rotation_speed = slider.value();
        }
    }

    for (radio_group, control) in &q_light_controls {
        let Ok(mut settings) = q_camera_control_settings.get_mut(control.camera_control) else {
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

fn update_camera_controls(
    time: Res<Time>,
    q_camera_controls: Query<(&CameraControl, Ref<CameraControlSettings>)>,
    mut ambient_light: ResMut<AmbientLight>,
    mut q_directional_light: Query<&mut DirectionalLight>,
    mut q_fog_settings: Query<&mut FogSettings>,
    mut q_transform: Query<&mut Transform>
) {
    for (camera_control, settings) in &q_camera_controls {
        let Ok(mut transform) = q_transform.get_mut(camera_control.camera()) else {
            continue;
        };

        if settings.do_rotate && settings.rotation_speed != 0.0 {
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

        if settings.is_changed() {
            let Ok(mut light) = q_directional_light.get_mut(camera_control.light()) else {
                continue;
            };
            let Ok(mut fog) = q_fog_settings.get_mut(camera_control.camera()) else {
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
struct ActiveCameraControls {
    camera_controls: HashMap<Entity, CameraControl>,
}

impl Default for ActiveCameraControls {
    fn default() -> Self {
        Self {
            camera_controls: HashMap::new(),
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, UiContext)]
#[reflect(Component)]
struct CameraControls {
    camera_control: Entity,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            camera_control: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for CameraControls {
    fn default_theme() -> Option<Theme<CameraControls>> {
        CameraControls::theme().into()
    }
}

impl CameraControls {
    pub fn theme() -> Theme<CameraControls> {
        let base_theme = PseudoTheme::deferred(None, CameraControls::primary_style);
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
    camera_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneRotationSpeedControl {
    camera_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneLightControl {
    camera_control: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
struct SpawnCameraControl;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct CameraControl {
    camera: Entity,
    light: Entity,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            camera: Entity::PLACEHOLDER,
            light: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct CameraControlSettings {
    do_rotate: bool,
    rotation_speed: f32,
    light: usize,
}

impl Default for CameraControlSettings {
    fn default() -> Self {
        Self {
            do_rotate: true,
            rotation_speed: 0.1,
            light: 0,
        }
    }
}

impl CameraControl {
    pub fn camera(&self) -> Entity {
        self.camera
    }

    pub fn light(&self) -> Entity {
        self.light
    }
}

pub trait UiCameraControlExt {
    fn camera_control(&mut self) -> UiBuilder<Entity>;
}

impl UiCameraControlExt for UiBuilder<'_, Entity> {
    fn camera_control(&mut self) -> UiBuilder<Entity> {
        let column = self
            .column(|_| {})
            .insert((Name::new("Camera Control"), SpawnCameraControl))
            .style() // Needed until UiImage stops depending on background color
            .background_color(Color::WHITE)
            .width(Val::Percent(100.0))
            .id();

        self.commands().ui_builder(column)
    }
}
