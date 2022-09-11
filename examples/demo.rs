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
            let mut dim_mappings = vec![vector![0.0, 0.0, 0.0]; MAX_NDIM as _];
            for i in 0..4 {
                dim_mappings[i] = Vector::unit(i as _);
            }

            Box::new(PolytopeDemo {
                polygons: vec![],
                ndim: 3,
                dim_mappings,

                auto_generate: false,

                cd: "4,3,3,3".to_string(),
                cd_error: false,
                poles: vec![Vector::unit(0)],
                arrows: vec![],

                camera_rot: Matrix::EMPTY_IDENT,
                active_axes: [0, 1, 2],
                pitch: 0.,
                yaw: 0.,
                w_offset: 4.,
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

    arrows: Vec<Vector<f32>>,

    camera_rot: Matrix<f32>,
    active_axes: [u8; 3],
    pitch: f32,
    yaw: f32,
    w_offset: f32,
}

impl PolytopeDemo {
    fn is_axis_flat(&self, axis: u8) -> bool {
        self.camera_rot.get(axis, axis) > 1. - 0.00001
    }

    // fn flatten_axis(&mut self, axis: u8) {
    //     for i in (0..self.camera_rot.ndim()) {
    //         if i == axis {
    //             *self.camera_rot.get_mut(i, i) = 1.;
    //         } else {
    //             *self.camera_rot.get_mut(i, axis) = 0.;
    //             *self.camera_rot.get_mut(axis, i) = 0.;
    //             let mag = self.camera_rot.col(i).mag();
    //             if mag == 0. {
    //                 *self.camera_rot.get_mut(i, i) = 1.;
    //             } else {
    //                 for j in (0..self.camera_rot.ndim()) {
    //                     *self.camera_rot.get_mut(i, j) /= mag;
    //                 }
    //             }
    //         }
    //     }
    // }

    fn rotate_camera(&mut self, axis0: u8, axis1: u8, angle: f32) {
        let cangle = angle.cos();
        let sangle = angle.sin();

        let mut m0 = Matrix::ident(MAX_NDIM);
        *m0.get_mut(axis0, axis0) = cangle;
        *m0.get_mut(axis0, axis1) = sangle;
        *m0.get_mut(axis1, axis0) = sangle;
        *m0.get_mut(axis1, axis1) = -cangle;
        self.camera_rot = &m0 * &self.camera_rot;
    }

    fn flatten_axis(&mut self, axis: u8) {
        let current = self.camera_rot.col(axis);
        let target = Vector::unit(axis);
        let tm = Matrix::from_outer_product(current, &target);
        let tm = &tm - &tm.transpose();
        let m0 = &(&Matrix::ident(MAX_NDIM) + &tm)
            + &((&tm * &tm).scale(1. / (1. + current.dot(target))));
        self.camera_rot = &m0 * &self.camera_rot;
    }
}

impl eframe::App for PolytopeDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut log = String::new();

        egui::SidePanel::new(egui::containers::panel::Side::Right, "right").show(ctx, |ui| {
            ui.label("W-Offset");
            ui.add(
                egui::DragValue::new(&mut self.w_offset)
                    .speed(0.01)
                    .fixed_decimals(1),
            );

            for axis in &mut self.active_axes {
                ui.horizontal(|ui| {
                    ui.selectable_value(axis, 0, "x");
                    ui.selectable_value(axis, 1, "y");
                    ui.selectable_value(axis, 2, "z");
                    ui.selectable_value(axis, 3, "w");
                    ui.selectable_value(axis, 4, "u");
                    ui.selectable_value(axis, 5, "v");
                    ui.selectable_value(axis, 6, "dim7");
                    ui.selectable_value(axis, 7, "dim8");
                });
            }
            ui.horizontal(|ui| {
                if ui.selectable_label(self.is_axis_flat(0), "x").clicked() {
                    self.flatten_axis(0);
                }
                if ui.selectable_label(self.is_axis_flat(1), "y").clicked() {
                    self.flatten_axis(1);
                }
                if ui.selectable_label(self.is_axis_flat(2), "z").clicked() {
                    self.flatten_axis(2);
                }
                if ui.selectable_label(self.is_axis_flat(3), "w").clicked() {
                    self.flatten_axis(3);
                }
                if ui.selectable_label(self.is_axis_flat(4), "u").clicked() {
                    self.flatten_axis(4);
                }
                if ui.selectable_label(self.is_axis_flat(5), "v").clicked() {
                    self.flatten_axis(5);
                }
                if ui.selectable_label(self.is_axis_flat(6), "dim7").clicked() {
                    self.flatten_axis(6);
                }
                if ui.selectable_label(self.is_axis_flat(7), "dim8").clicked() {
                    self.flatten_axis(7);
                }
            });
            if ui.button("Reset Camera").clicked() {
                self.camera_rot = Matrix::EMPTY_IDENT;
            }

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
                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.poles.push(Vector::EMPTY);
                    }
                    if ui.button("-").clicked() && self.poles.len() > 1 {
                        self.poles.pop();
                    }
                });
                for p in &mut self.poles {
                    vector_edit(ui, p, self.ndim);
                }

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
                        self.arrows = cd.mirrors().iter().map(|v| v.0.clone()).collect();
                        let m = Matrix::from_cols(cd.mirrors().iter().rev().map(|v| &v.0))
                            .inverse()
                            .transpose();
                        let group = cd.generators();
                        for p in &mut self.poles {
                            p.truncate(self.ndim);
                        }
                        let poles = self
                            .poles
                            .iter()
                            .map(|v| m.transform(v))
                            .collect::<Vec<_>>();
                        self.arrows.extend_from_slice(&poles);
                        self.polygons = shape_geom(self.ndim, &group, &poles);
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
                    vector_edit(ui, v, 4);
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
            let r = egui::plot::Plot::new("polygon_plot")
                .data_aspect(1.0)
                .allow_boxed_zoom(false)
                .show(ui, |plot_ui| {
                    let ndrot = &Matrix::from_cols(self.dim_mappings.clone()) * &self.camera_rot;
                    // let rot = cgmath::Matrix3::from_angle_x(cgmath::Rad(self.pitch))
                    //     * cgmath::Matrix3::from_angle_y(cgmath::Rad(self.yaw));
                    for (i, p) in self.polygons.iter().enumerate() {
                        plot_ui.polygon(
                            egui::plot::Polygon::new(egui::plot::Values::from_values_iter(
                                p.verts
                                    .iter()
                                    .map(|p| {
                                        let mut v = ndrot.transform(p);
                                        let w = v[3] + self.w_offset;
                                        v = v / w;
                                        cgmath::point3(v[0], v[1], v[2])
                                    })
                                    .map(|xy| egui::plot::Value::new(xy.x, xy.y)),
                            ))
                            .name(i),
                        );
                    }
                    plot_ui.arrows(egui::plot::Arrows::new(
                        egui::plot::Values::from_values_iter(
                            vec![egui::plot::Value::new(0, 0); self.arrows.len()].into_iter(),
                        ),
                        egui::plot::Values::from_values_iter(
                            self.arrows
                                .iter()
                                .map(|p| {
                                    let mut v = ndrot.transform(p);
                                    let w = v[3] + self.w_offset;
                                    v = v / w;
                                    cgmath::point3(v[0], v[1], v[2])
                                })
                                .map(|xy| egui::plot::Value::new(xy.x, xy.y)),
                        ),
                    ))
                });
            if r.response.dragged_by(egui::PointerButton::Secondary) {
                let egui::Vec2 { x, y } = r.response.drag_delta();
                let dx = x / 100.;
                let dy = y / 100.;

                let [a0, a1, a2] = self.active_axes;

                self.rotate_camera(a0, a2, dx);
                self.rotate_camera(a1, a2, dy);
            }
        });
    }
}

fn vector_edit(ui: &mut egui::Ui, v: &mut Vector<f32>, ndim: u8) {
    ui.horizontal(|ui| {
        for i in 0..ndim {
            ui.add(
                egui::DragValue::new(&mut v[i])
                    .speed(0.01)
                    .fixed_decimals(1),
            )
            .on_hover_text(format!("Dim {i}"));
        }
    });
}
