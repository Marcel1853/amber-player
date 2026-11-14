// Befehle, die die Oberfläche beim Android-Teil aufrufen darf.
const COMMANDS: &[&str] = &[
    "update_media",
    "stop_media",
    "ignore_battery_optimizations",
    "take_pair_link",
    "get_fullscreen",
    "set_fullscreen",
    "open_url",
    "move_to_back",
    "share_file",
    "take_shared_file",
    "check_permissions",
    "request_permissions",
    "register_listener",
    "remove_listener",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).android_path("android").build();
}
