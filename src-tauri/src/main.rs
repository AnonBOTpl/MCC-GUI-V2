// Na Windows w wersji release nie otwieramy dodatkowego okna konsoli.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mcc_gui_v2_lib::run()
}
