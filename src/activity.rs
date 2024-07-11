use std::fmt;

use bevy::prelude::*;

use crate::framework::*;

pub mod new_project;

/// The activity service provides a persistent context for a particular editor function like
/// "mmanage an asset" or "preview a scene." It can track multiple instances of the same
/// activity and manages a stack of all activities.
#[derive(Resource, Debug)]
pub struct ActivityService {
    pub current_activity: Box<dyn Activity>,
}

impl ActivityService {
    pub fn start(&mut self) -> EditorId {
        self.current_activity.start()
    }

    pub fn stop(&mut self) {
        self.current_activity.stop();
    }

    pub fn restart(&mut self, mut activity: impl Activity) -> EditorId {
        activity.stop();
        activity.start()
    }
}

impl Default for ActivityService {
    fn default() -> Self {
        Self { current_activity: Box::new(DefaultActivity) }
    }
}

pub trait Activity: Reflect + fmt::Debug {
    fn start(&mut self) -> EditorId;
    fn stop(&mut self);
}

#[derive(Reflect, Debug, Default)]
pub struct DefaultActivity;

impl Activity for DefaultActivity {
    fn start(&mut self) -> EditorId {
        EditorId::default()
    }

    fn stop(&mut self) {}
}
