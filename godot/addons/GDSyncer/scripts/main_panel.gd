@tool
extends Control


var server_address_setting: String
var server_password_setting: String

signal start_called


func _on_save_config_button_pressed() -> void:
	set_setting(
		server_address_setting, 
		$Tabs/Server/Settings/GridContainer/AddressEdit.text
	)
	
	set_setting(
		server_password_setting, 
		$Tabs/Server/Settings/GridContainer/PasswordEdit.text
	)
	
	print("[GDSyncer] Settings saved.")


func set_setting(setting_path: String, setting_value: String) -> void:
	ProjectSettings.set_setting(setting_path, setting_value)


func set_vars(
	server_address_path: String,
	server_password_path: String
) -> void:
	server_address_setting = server_address_path
	server_password_setting = server_password_path


func _on_start_button_pressed():
	emit_signal("start_called")
