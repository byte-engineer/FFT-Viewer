use csv::{Reader, Writer};
use eframe::egui::{self, Vec2b};
use egui_plot::{Line, Plot, PlotPoints, Points};
use rand::Rng;
use rustfft::{FftPlanner, num_complex::Complex};
use std::{f64::consts::PI, process::exit};
use puffin::{profile_function, profile_scope};


const RECORD_LENGTH: usize = 10000;

pub struct RustyApp {
    frame_count: u64,
    fft_points: Vec<Complex<f64>>,
    line_chked: bool,
    enable_fft: bool,
    enable_points: bool,
    sine_freq: f64,
    auto_bounds: bool,
    waveform: WaveForm,
}


struct WaveForm {
    points: Vec<Point>,
    sampling_rate: usize,
    record_length: usize,
}

#[derive(Clone, Copy, Default)]
struct Point {
    x: f64,
    y: f64,
}

impl RustyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.global_style()).clone();

        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.visuals.dark_mode = true;
        style.visuals.window_corner_radius = 8.0.into();

        cc.egui_ctx.set_global_style(style);

        Self {
            frame_count: 0,
            line_chked: true,
            fft_points: vec![Complex { re: 0.0, im: 0.0 }; RECORD_LENGTH],
            enable_fft: true,
            enable_points: true,
            sine_freq: 10.0,
            auto_bounds: false,
            waveform: WaveForm { 
                points: Vec::new(), sampling_rate: 1000, record_length: RECORD_LENGTH
            }
        }
    }

    fn update_points(&mut self) {
        profile_function!("Update Points");
        let mut rng = rand::rng();

        self.waveform.points.resize(self.waveform.record_length, Point::default());
        self.fft_points.resize(self.waveform.record_length, Complex::default());

        for (i, p) in self.waveform.points.iter_mut().enumerate() {
            p.x = i as f64;
            p.y = rng.random_range(-0.5f64..=0.5f64)
            + ((i as f64) * 2.0 * PI * rng.random_range(-10.0f64..=self.waveform.sampling_rate as f64)/self.waveform.sampling_rate as f64).sin()
                + ((i as f64) * 2.0 * PI * self.sine_freq/self.waveform.sampling_rate as f64).sin();
        }
    }
}




impl eframe::App for RustyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

        puffin::GlobalProfiler::lock().new_frame();
        self.frame_count += 1;

        // Update simulation
        self.update_points();
        let fps = (1.0 / ui.input(|i| i.stable_dt)).round();

        
        ui.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                // ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                exit(0);
            }
        });

        
        egui::CentralPanel::default().show(ui, |ui| {
            
            // let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
    
    
            // for file in dropped_files {
            //     if let Some(path) = file.path {
            //         println!("Dropped: {}", path.display());
    
            //         // Load your file here
            //         // self.load_file(&path);
            //     }
            // }
    
            // let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
    
            // if hovering {
            //     egui::Area::new("drop_overlay".into())
            //         .fixed_pos(egui::pos2(20.0, 20.0))
            //         .show(ui.ctx(), |ui| {
            //             ui.label("Drop files here");
            //         });
            // }



            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(self.fft_points.len());
            self.fft_points = self.waveform.points
            .iter()
                .map(|x| Complex { re: x.y, im: 0.0 })
                .collect();

            {
                puffin::profile_scope!("Process FFT");
                fft.process(&mut self.fft_points);
            }

            let plot_points_for_scatter =
                PlotPoints::from_iter(self.waveform.points.iter().map(|p| [p.x, p.y]));

            let plot_points_for_lines =
                PlotPoints::from_iter(self.waveform.points.iter().map(|p| [p.x, p.y]));

            let plot_points_for_fft = PlotPoints::from_iter(
                self.fft_points[0..self.fft_points.len()/2]
                    .iter()
                    .enumerate()
                    .map(|(i, p)| [i as f64, (p.norm()/RECORD_LENGTH as f64).log10()]),
            );


            let scatter = Points::new("Particles", plot_points_for_scatter)
                .radius(3.0)
                .color(egui::Color32::from_rgb(232, 160, 124));

            let line = Line::new("Line", plot_points_for_lines)
                .color(egui::Color32::from_rgb(232, 160, 124));

            let fft_line = Line::new("FFT", plot_points_for_fft);

            ui.horizontal(|ui| {

                profile_scope!("Building UI");
                main_frame(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Frame: {}", self.frame_count));
                        ui.label(format!("FPS  : {}", fps));
                        // if ui.button("Save CSV").clicked() {
                        //     save_points("data.csv", &self.waveform.points);
                        // }

                        ui.checkbox(&mut self.auto_bounds, "Auto Bounds");
                    });
                });

                main_frame(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.sine_freq, 0.1..=10000.0)
                                .logarithmic(false)
                                .text("Frequency"),
                        );

                        ui.add(
                            egui::Slider::new(&mut self.waveform.sampling_rate, 2..=10000)
                                .text("Sampling Rate"),
                        );
            
                        ui.add(
                            egui::Slider::new(&mut self.waveform.record_length, 10..=80000)
                                .logarithmic(true)
                                .text("Record length")
                        );
            
                    });
                });
    
    
                main_frame(ui, |ui|{  
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.enable_fft, "Draw FFT.");
                        ui.checkbox(&mut self.enable_points, "Draw points.");
                        if self.enable_points {
                            ui.indent("points settings", |ui| {
                                ui.checkbox(&mut self.line_chked, "Draw Lines.");
                            });
                        }
                    })
                });
            });


            Plot::new("Points & fft Plot")
                .x_axis_formatter(|mark, _range| {
                    match mark.value {
                        x if x == 0.0 => "DC".to_owned(),
                        x => format!(
                            "{:.1} Hz\n{:.1} sec",
                            x*(self.waveform.sampling_rate as f64)/(self.waveform.record_length as f64),
                            x/(self.waveform.sampling_rate as f64)),
                    }
                })
                .y_axis_formatter(|mark, _range| {
                    match mark.value {
                        x => format!("{} db", (x as f64))
                    }
                })

                .show(ui, |plot_ui| {
                    puffin::profile_scope!("Plotting");
                    if self.enable_points {
                        if self.line_chked {
                            plot_ui.line(line);
                        } else {
                            plot_ui.points(scatter);
                        }
                    }

                    if self.auto_bounds {
                        plot_ui.set_auto_bounds(Vec2b::new(true, true));
                    }

                    if self.enable_fft {
                        plot_ui.line(fft_line);
                    }

                }); // plot .show()
        }); // Central panel

        ui.request_repaint();
    }
}

#[allow(unused)]
fn fetch_points(path: &str) -> Vec<Point> {
    let mut reader = Reader::from_path(path).unwrap();

    reader
        .records()
        .map(|r| {
            let r = r.unwrap();
            Point {
                x: r[0].parse().unwrap(),
                y: r[1].parse().unwrap(),
            }
        })
        .collect()
}

#[allow(unused)]
fn save_points(path: &str, points: &[Point]) {
    let mut writer = Writer::from_path(path).unwrap();

    for p in points {
        writer
            .write_record(&[p.x.to_string(), p.y.to_string()])
            .unwrap();
    }

    writer.flush().unwrap();
}



fn main_frame<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    puffin::profile_function!("main_frame");
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
        .inner_margin(5.0)
        .outer_margin(1.0)
        .show(ui, |ui|{
            ui.set_min_height(85.0);
            add_contents(ui)
        })
}