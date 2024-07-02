// ideally we have a signals abstraction here that is suitable to eventually become bevy_signals

// this would require an API that various projects like haalka (futures-signals), bevy_dioxus,
// bevy_quill, bevy_rx, polako, bevy_lazy_signals, etc could be plugged into instead of the default

// it may make sense to model the default directly on futures-signals because it has wide adoption in Rust

// the idea is to be able to provide a surface API like the TC39 Signals proposal

// hopefully this will provide familiar patterns to UI developers coming in from TypeScript and HTML/CSS
