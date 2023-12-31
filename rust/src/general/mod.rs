use godot::{
    engine::{global::PropertyHint, EditorPlugin, EditorPluginVirtual, ProjectSettings, Window},
    prelude::*,
};
use tokio::runtime::{Builder, Runtime};
// use tokio::runtime::{Builder, Runtime};

use crate::{client::ClientWrapper, server::ServerWrapper};

const SERVER_ADDRESS: &str = "gdsyncer/server_settings/host";
const SERVER_ADDRESS_DEFAULT: &str = "0.0.0.0:8008";
const SERVER_PASSWORD: &str = "gdsyncer/server_settings/password";
const SERVER_PASSWORD_DEFAULT: &str = "";

#[derive(Debug, GodotClass)]
#[class(tool, base=EditorPlugin, editor_plugin)]
struct Entrypoint {
    server: Gd<ServerWrapper>,
    client: Gd<ClientWrapper>,
    runtime: Runtime,
    main_panel_windowed: Gd<Window>,
    #[base]
    editor_plugin: Base<EditorPlugin>,
}

#[godot_api]
impl EditorPluginVirtual for Entrypoint {
    fn init(editor_plugin: Base<EditorPlugin>) -> Self {
        let mut server = Gd::<ServerWrapper>::with_base(ServerWrapper::new);
        server.set_name("Server".into());
        let mut client = Gd::<ClientWrapper>::with_base(ClientWrapper::new);
        client.set_name("Client".into());

        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let main_panel_windowed =
            load::<PackedScene>("res://addons/GDSyncer/scenes/main_panel_windowed.tscn")
                .instantiate_as::<Window>();

        let res = Self {
            server,
            client,
            runtime,
            main_panel_windowed,
            editor_plugin,
        };

        res.init_properties();

        res
    }

    fn get_plugin_name(&self) -> GodotString {
        GodotString::from("GDSyncer")
    }

    fn enter_tree(&mut self) {
        godot_print!("GDSyncer initialized.");
    }

    fn exit_tree(&mut self) {
        self.editor_plugin
            .get_editor_interface()
            .unwrap()
            .get_base_control()
            .unwrap()
            .remove_child(self.main_panel_windowed.clone().upcast::<Node>());
        self.main_panel_windowed.queue_free();
        self.server.bind().shutdown_sync();

        let plugin_name = self.get_plugin_name();
        self.editor_plugin.remove_tool_menu_item(plugin_name);
    }

    fn ready(&mut self) {
        self.init_interface();
    }
}

#[godot_api]
impl Entrypoint {
    #[signal]
    fn code_changed();

    #[func]
    pub fn init_properties(&self) {
        Self::set_setting_with_value(
            SERVER_ADDRESS.into(),
            SERVER_ADDRESS_DEFAULT.to_variant(),
            Some(dict! {
                "name": SERVER_ADDRESS,
                "type": VariantType::String as i32,
                "hint": PropertyHint::PROPERTY_HINT_NONE,
                "hint_string": ""
            }),
        );

        Self::set_setting_with_value(
            SERVER_PASSWORD.into(),
            SERVER_PASSWORD_DEFAULT.to_variant(),
            Some(dict! {
                "name": SERVER_PASSWORD,
                "type": VariantType::String as i32,
                "hint": PropertyHint::PROPERTY_HINT_NONE,
                "hint_string": ""
            }),
        );

        let mut project_settings = ProjectSettings::singleton();
        let save_result = project_settings.save();
        if save_result.ord() > 0 {
            godot_error!(
                "[GSyncer] An error occured while trying to save config: {:?}",
                save_result
            );
        };
    }

    pub fn set_setting_with_value(setting: String, value: Variant, hint: Option<Dictionary>) {
        let mut project_settings = ProjectSettings::singleton();
        let setting: GodotString = setting.into();

        if !project_settings.has_setting(setting.clone()) {
            project_settings.set_setting(setting.clone(), Variant::from(value.clone()));
        }

        if let Some(hint_unwrapped) = hint {
            if !hint_unwrapped.is_empty() {
                project_settings.add_property_info(hint_unwrapped);
            }
        }

        project_settings.set_initial_value(setting, value);
    }

    pub fn get_main_panel(&self) -> Gd<Node> {
        self.main_panel_windowed
            .find_child("MainPanel".into())
            .unwrap()
    }

    #[func]
    pub fn connect_client(&mut self) {
        let client = self.client.bind();

        if client.is_connected() {
            godot_warn!("Client already running!");
        }

        client.connect_sync("http://127.0.0.1:8008".into());
    }

    #[func]
    pub fn init_interface(&mut self) {
        let mut main_panel = self.get_main_panel();
        main_panel.call(
            "set_vars".into(),
            &[SERVER_ADDRESS.to_variant(), SERVER_PASSWORD.to_variant()],
        );

        let server_callable = self.server.callable("start_or_shutdown_sync");
        main_panel.connect("create_server".into(), server_callable);

        let client_callable = self.editor_plugin.callable("connect_client");
        main_panel.connect("create_client".into(), client_callable);

        let plugin_name = self.get_plugin_name();

        self.main_panel_windowed.set_title(plugin_name.clone());

        let close_callable = self.main_panel_windowed.callable("hide");
        self.main_panel_windowed
            .connect("close_requested".into(), close_callable);

        self.main_panel_windowed.hide();

        self.editor_plugin
            .get_editor_interface()
            .unwrap()
            .get_base_control()
            .unwrap()
            .add_child(self.main_panel_windowed.clone().upcast::<Node>());

        let open_callable = self.main_panel_windowed.callable("show");

        self.editor_plugin
            .add_tool_menu_item(plugin_name, open_callable);
    }
}
