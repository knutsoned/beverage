// this is where the archetype / blueprint / construct / prefab / prototype / template goes

use bevy::prelude::*;

/// The construct service manages templates and instances for prefabs, blueprints, etc.
#[derive(Resource, Default, Debug)]
pub struct ConstructService {}
