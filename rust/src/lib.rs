use godot::{init::EditorRunBehavior, prelude::*};

mod client;
mod general;
mod server;
mod utils;

struct GDSyncer;

#[gdextension]
unsafe impl ExtensionLibrary for GDSyncer {
    fn editor_run_behavior() -> EditorRunBehavior {
        EditorRunBehavior::ToolClassesOnly
    }
}
