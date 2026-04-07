pub fn generate_window_interface(context: &egui::Context) {
    egui::Window::new("Налаштування рендеру").show(&context, |ui| {
        ui.label("Привіт, egui працює!");
        if ui.button("Натисни мене").clicked() {
            println!("Кнопка натиснута!");
        }
    });
}
