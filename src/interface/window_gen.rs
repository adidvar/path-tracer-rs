use crate::RenderSettings;

pub fn generate_window_interface(context: &egui::Context, settings: &mut RenderSettings) {
    egui::Window::new("Налаштування рендеру").show(context, |ui| {
        ui.heading("Параметри зображення");

        ui.checkbox(&mut settings.enable_tonemapping, "Увімкнути Tonemapping");
        ui.checkbox(&mut settings.enable_gamma, "Увімкнути Gamma Correction");

        ui.separator();

        ui.add(egui::Slider::new(&mut settings.time, 0.0..=100.0).text("Час (с)"));

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Статус:");
            ui.colored_label(egui::Color32::GREEN, "Рендеринг активний");
        });

        if ui.button("Скинути налаштування").clicked() {
            *settings = RenderSettings::default();
        }
    });
}
