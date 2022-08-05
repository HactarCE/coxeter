use cgmath::Transform;
use eframe::egui;
use itertools::Itertools;
use symmetries::*;

const MAX_NDIM: u8 = 8;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Polytope generator demo",
        options,
        Box::new(|_cc| {
            let mut dim_mappings = vec![vector![1.0, 0.0, 0.0]; MAX_NDIM as _];
            for i in 0..3 {
                dim_mappings[i] = Vector::unit(i as _);
            }

            Box::new(PolytopeDemo {
                polygons: vec![],
                ndim: 3,
                dim_mappings,

                auto_generate: false,

                cd: "3".to_string(),
                cd_error: false,
                poles: vec![Vector::unit(0)],
            })
        }),
    );
}

#[derive(Debug)]
struct PolytopeDemo {
    polygons: Vec<Polygon>,
    ndim: u8,
    dim_mappings: Vec<Vector<f32>>,

    auto_generate: bool,

    cd: String,
    cd_error: bool,
    poles: Vec<Vector<f32>>,
}
impl eframe::App for PolytopeDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pitch_id = egui::Id::new("pitch");
        let yaw_id = egui::Id::new("yaw");

        let mut log = String::new();

        egui::SidePanel::new(egui::containers::panel::Side::Right, "right").show(ctx, |ui| {
            ui.label("Pitch");
            let mut pitch = ui.data().get_persisted(pitch_id).unwrap_or(0.0);
            ui.drag_angle(&mut pitch);
            ui.data().insert_persisted(
                pitch_id,
                pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2),
            );

            ui.label("Yaw");
            let mut yaw = ui.data().get_persisted(yaw_id).unwrap_or(0.0);
            ui.drag_angle(&mut yaw);
            ui.data()
                .insert_persisted(yaw_id, yaw.rem_euclid(std::f32::consts::TAU));

            ui.separator();
            ui.add(
                egui::DragValue::new(&mut self.ndim)
                    .clamp_range(1..=MAX_NDIM)
                    .speed(0.1),
            );
            if ui.button("Generate cube").clicked() {
                self.polygons = PolytopeArena::new_cube(self.ndim, 1.0).polygons();
            }
            ui.collapsing("Coxeter diagram", |ui| {
                ui.text_edit_singleline(&mut self.cd);

                ui.strong("Base facets");
                for p in &mut self.poles {
                    vector_edit(ui, p, self.ndim);
                }
                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.poles.push(Vector::EMPTY);
                    }
                    if ui.button("-").clicked() && self.poles.len() > 1 {
                        self.poles.pop();
                    }
                });

                if ui.button("Generate!").clicked() || self.auto_generate {
                    self.cd_error = false;
                    let xs = self
                        .cd
                        .split(',')
                        .map(|s| s.trim().parse().unwrap_or(0))
                        .collect_vec();
                    if xs.iter().any(|&x| x <= 1) {
                        self.cd_error = true;
                    } else {
                        let cd = CoxeterDiagram::with_edges(xs);
                        self.ndim = cd.ndim();
                        let group = cd.generators();
                        for p in &mut self.poles {
                            p.truncate(self.ndim);
                        }
                        self.polygons = shape_geom(self.ndim, &group, &self.poles);
                    }
                }
                ui.checkbox(&mut self.auto_generate, "Auto generate");
                ui.colored_label(egui::Color32::RED, if self.cd_error { "error" } else { "" });
            });

            ui.separator();
            for (dim, v) in self.dim_mappings.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Dim {dim}"));
                    if ui.button("N").clicked() {
                        if v.dot(&*v) != 0.0 {
                            *v = &*v * (1.0 / v.dot(&*v).sqrt());
                        }
                    }
                    vector_edit(ui, v, 3);
                });
            }

            ui.separator();
            ui.label("Log:");
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut log).interactive(false));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Polytope generator demo");

            let pitch: f32 = ui.data().get_persisted(pitch_id).unwrap_or(0.0);
            let yaw: f32 = ui.data().get_persisted(yaw_id).unwrap_or(0.0);
            egui::plot::Plot::new("polygon_plot")
                .data_aspect(1.0)
                .show(ui, |plot_ui| {
                    let ndrot = Matrix::from_cols(self.dim_mappings.clone());
                    let rot = cgmath::Matrix3::from_angle_x(cgmath::Rad(pitch))
                        * cgmath::Matrix3::from_angle_y(cgmath::Rad(yaw));
                    for (i, p) in self.polygons.iter().enumerate() {
                        plot_ui.polygon(
                            egui::plot::Polygon::new(egui::plot::Values::from_values_iter(
                                p.verts
                                    .iter()
                                    .map(|p| {
                                        rot.transform_point({
                                            let v = ndrot.transform(p);
                                            cgmath::point3(v[0], v[1], v[2])
                                        })
                                    })
                                    .map(|xy| egui::plot::Value::new(xy.x, xy.y)),
                            ))
                            .name(i),
                        );
                    }
                });
        });
    }
}

fn vector_edit(ui: &mut egui::Ui, v: &mut Vector<f32>, ndim: u8) {
    ui.horizontal(|ui| {
        for i in 0..ndim {
            ui.add(egui::DragValue::new(&mut v[i]).speed(0.01))
                .on_hover_text(format!("Dim {i}"));
        }
    });
}
