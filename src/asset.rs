use bevy::{ prelude::*, utils::HashMap };

use crate::framework::*;

/// The asset service provides integration between internal metadata used by the editor and the
/// regular Bevy asset infrastructure.
#[derive(Resource, Default, Debug)]
pub struct AssetService {
    atlas_map: HashMap<EditorId, EditorAtlas>,
}

impl AssetService {
    pub fn get_atlas(&mut self, id: EditorId) -> EditorAtlas {
        if let Some(atlas) = self.atlas_map.get(&id) {
            return atlas.clone();
        }

        let atlas = EditorAtlas { id: id.clone() };
        self.atlas_map.insert(id.clone(), atlas.clone());
        atlas.clone()
    }
}
