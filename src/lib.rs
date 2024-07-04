pub mod a11y;
pub mod construct;
pub mod framework;
pub mod input;
pub mod locale;
pub mod layout;
pub mod logging;
pub mod remote;
pub mod router;
pub mod setup;
pub mod signals;
pub mod theme;
pub mod undo;
pub mod widget;

// plugins will want to have the domain objects available
pub mod prelude {
    pub use crate::framework::*;
}
