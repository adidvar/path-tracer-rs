use crate::AppSettings;

pub fn generate_window_interface(context: &egui::Context, settings: &mut AppSettings) {
    egui::Window::new("Render Settings").show(context, |ui| {
        ui.heading("Image Parameters");

        ui.checkbox(&mut settings.enable_tonemapping, "Enable Tonemapping");
        ui.checkbox(&mut settings.enable_gamma, "Enable Gamma Correction");

        ui.separator();

        ui.add(egui::Slider::new(&mut settings.time, 0.0..=100.0).text("Time (s)"));
        if ui
            .add(
                egui::Slider::new(&mut settings.global_light_intensity, 0.0..=5.0)
                    .text("Light Intensity"),
            )
            .changed()
        {
            settings.camera_dirty = true;
        }
        if ui
            .add(egui::Slider::new(&mut settings.max_bounces, 1..=16).text("Max Bounces"))
            .changed()
        {
            settings.camera_dirty = true;
        }
        if ui
            .add(egui::Slider::new(&mut settings.rays_per_pixel, 1..=100).text("Rays Per Pixel"))
            .changed()
        {
            settings.camera_dirty = true;
        }
        if ui
            .checkbox(&mut settings.use_msaa, "Enable MSAA (Anti-aliasing)")
            .changed()
        {
            settings.camera_dirty = true;
        }

        ui.separator();
        ui.heading("Camera");

        let cam = &mut settings.camera;
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Position X:");
            changed |= ui
                .add(egui::DragValue::new(&mut cam.position[0]).speed(0.05))
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Position Y:");
            changed |= ui
                .add(egui::DragValue::new(&mut cam.position[1]).speed(0.05))
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Position Z:");
            changed |= ui
                .add(egui::DragValue::new(&mut cam.position[2]).speed(0.05))
                .changed();
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Pitch (°):");
            changed |= ui
                .add(
                    egui::DragValue::new(&mut cam.rotation_angles[0])
                        .speed(0.5)
                        .range(-89.0..=89.0),
                )
                .changed();
        });
        ui.horizontal(|ui| {
            ui.label("Yaw (°):");
            changed |= ui
                .add(egui::DragValue::new(&mut cam.rotation_angles[1]).speed(0.5))
                .changed();
        });

        ui.separator();

        changed |= ui
            .add(egui::Slider::new(&mut cam.fov, 10.0..=120.0).text("FOV (°)"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut cam.aperture, 0.0..=0.5).text("Aperture"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut cam.focus_distance, 0.1..=50.0).text("Focus Distance"))
            .changed();

        if changed {
            settings.camera_dirty = true;
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Status:");
            ui.colored_label(egui::Color32::GREEN, "Rendering active");
        });

        if ui.button("Reset settings").clicked() {
            *settings = AppSettings::default();
            settings.camera_dirty = true;
        }
    });

    egui::Window::new("Global Statistics").show(context, |ui| {
        ui.heading("Performance");
        ui.label(format!(
            "Render step time (CPU Submit): {:.2} ms",
            settings.render_time_ms
        ));
    });
}
