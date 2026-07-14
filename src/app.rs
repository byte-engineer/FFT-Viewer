use csv::{Reader, Writer};
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use rand::Rng;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f64::consts::PI;

const RECORD_LENGTH: usize = 100;

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
            sine_freq: 0.00010,
            auto_bounds: true,
            waveform: WaveForm { 
                points: Vec::new(), sampling_rate: 100, record_length: RECORD_LENGTH
            }
        }
    }

    fn update_points(&mut self) {
        let mut rng = rand::rng();

        self.waveform.points.resize(self.waveform.record_length, Point::default());
        self.fft_points.resize(self.waveform.record_length, Complex::default());

        for (i, p) in self.waveform.points.iter_mut().enumerate() {
            p.x = i as f64;
            p.y = rng.random_range(-0.01f64..=0.01f64)
                + (((i as f64) * 2.0 * PI * self.sine_freq/self.waveform.sampling_rate as f64).sin()).signum() * 10.0
                + ((i as f64) * 10.0 * PI * self.sine_freq/self.waveform.sampling_rate as f64).sin()
        }
    }
}

impl eframe::App for RustyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {

        self.frame_count += 1;

        // Update simulation
        self.update_points();

        egui::CentralPanel::default().show(ui, |ui| {
            if ui.button("Save CSV").clicked() {
                save_points("data.csv", &self.waveform.points);
            }

            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(self.fft_points.len());
            self.fft_points = self.waveform.points
                .iter()
                .map(|x| Complex { re: x.y, im: 0.0 })
                .collect();

            fft.process(&mut self.fft_points);

            ui.label(format!("Frame: {}", self.frame_count));

            ui.add(
                egui::Slider::new(&mut self.sine_freq, 0.001..=1000.0)
                    .logarithmic(true)
                    .text("Frequency"),
            );

            let response = ui.add(
                egui::Slider::new(&mut self.waveform.record_length, 10..=1000000)
                    .logarithmic(true)
                    .text("Record length")
            );

            ui.add(
                egui::Slider::new(&mut self.waveform.sampling_rate, 2..=1000)
                    .text("Sampling Rate"),
            );

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

            ui.checkbox(&mut self.line_chked, "Draw Lines.");
            ui.checkbox(&mut self.enable_fft, "Draw FFT.");
            ui.checkbox(&mut self.enable_points, "Draw points.");
            ui.checkbox(&mut self.auto_bounds, "auto bounds.");

            Plot::new("Combined Plot")
                .show_x(true)
                .show_y(true)
                .show(ui, |plot_ui| {

                    if self.enable_points {
                        if self.line_chked {
                            plot_ui.line(line);
                        } else {
                            plot_ui.points(scatter);
                        }
                    }

                    if self.enable_fft {
                        plot_ui.line(fft_line);
                    }
                });
        });

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

fn save_points(path: &str, points: &[Point]) {
    let mut writer = Writer::from_path(path).unwrap();

    for p in points {
        writer
            .write_record(&[p.x.to_string(), p.y.to_string()])
            .unwrap();
    }

    writer.flush().unwrap();
}
