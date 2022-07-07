use num_bigint::BigInt;
use std::fmt::LowerHex;
use substring::Substring;
use svg::node::element::path::Data;
use svg::node::element::{Element, Path, Text, SVG};
use svg::{Document, Node};

fn rect(x0: u32, y0: u32, x1: u32, y1: u32, color: &str) -> Path {
    let data = Data::new()
        .move_to((x0, y0))
        .line_to((x1, y0))
        .line_to((x1, y1))
        .line_to((x0, y1))
        .close();
    let path = Path::new()
        .set("fill", color)
        .set("stroke", "none")
        .set("stroke-width", 0)
        .set("d", data);
    path
}

fn line(x0: u32, y0: u32, x1: u32, y1: u32, color: &str) -> Path {
    let data = Data::new().move_to((x0, y0)).line_to((x1, y1));
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", color)
        .set("stroke-width", 1)
        .set("d", data);
    path
}

#[derive(Clone, PartialEq)]
struct TimedValue<T: PartialEq + Clone> {
    time: u64,
    value: T,
}

#[derive(Clone, PartialEq)]
struct Interval<T: PartialEq + Clone> {
    start_time: u64,
    end_time: u64,
    value: T,
    start_x: f64,
    end_x: f64,
    label: String,
}

impl<T: PartialEq + Clone> Interval<T> {
    pub fn is_empty(&self) -> bool {
        self.end_time == self.start_time
    }
}

#[derive(Clone, Debug)]
struct TimeView {
    start_time: u64,
    end_time: u64,
    pixel_scale: f64,
}

impl TimeView {
    pub fn map(&self, time: u64) -> f64 {
        (self.start_time.max(time).min(self.end_time) - self.start_time) as f64 * self.pixel_scale
    }
    pub fn intervals<T: PartialEq + Clone + LowerHex>(
        &self,
        vals: &[TimedValue<T>],
    ) -> Vec<Interval<T>> {
        vals.windows(2)
            .map(|x| {
                let end_x = self.map(x[1].time);
                let start_x = self.map(x[0].time);
                let label_max = ((end_x - start_x) / 6.0).round() as usize;
                let mut label = format!("0h{:x}", x[0].value);
                if label.len() > label_max {
                    if label_max <= 3 {
                        label = format!("!");
                    } else {
                        label = format!("{}+", label.substring(0, label_max - 1));
                    }
                }
                if label_max <= 3 {
                    label = "...".to_string()
                } else {
                }
                Interval {
                    start_time: x[0].time,
                    end_time: x[1].time,
                    value: x[0].value.clone(),
                    start_x,
                    end_x,
                    label,
                }
            })
            .collect()
    }
}

fn changes<T: PartialEq + Clone>(vals: &[TimedValue<T>]) -> Vec<TimedValue<T>> {
    if vals.is_empty() {
        vec![]
    } else {
        let mut prev = vals[0].clone();
        let mut ret = vec![prev.clone()];
        for val in vals {
            if val.value.ne(&prev.value) {
                ret.push(val.clone());
                prev = val.clone();
            }
        }
        ret
    }
}

fn make_clock(period: u64) -> Vec<TimedValue<bool>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: x % 2 == 0,
        })
        .collect()
}

fn make_linear_counter(period: u64) -> Vec<TimedValue<BigInt>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: x.into(),
        })
        .collect()
}

fn make_counter(period: u64) -> Vec<TimedValue<BigInt>> {
    (0..1000)
        .map(|x| TimedValue {
            time: period * x,
            value: (x * 10000 + x * x * 100000).into(),
        })
        .collect()
}

struct DisplayMetrics {
    signal_width: u32,
    signal_height: u32,
    timescale_height: u32,
    tick_half_height: u32,
    timescale_midline: u32,
    canvas_width: u32,
    canvas_height: u32,
    shim: u32,
    label_size: u32,
    min_time: u64,
    max_time: u64,
}

impl Default for DisplayMetrics {
    fn default() -> Self {
        Self {
            signal_width: 200,
            signal_height: 20,
            timescale_height: 45,
            tick_half_height: 6,
            timescale_midline: 20,
            canvas_width: 1000,
            canvas_height: 400,
            shim: 5,
            label_size: 10,
            min_time: 40,
            max_time: 102,
        }
    }
}

// We want major_tick_delt * 10 ~= max_time
// We also want major_tick_delt = [1, 2, 5] * 10^x
// So major_tick_delt = [1, 2, 5] * 10^x * 10 = max_time
//                      [1, 2, 5] * 10^{x+1} = max_time
//                    [0, log10(2), log10(5)] + (x+1) = log10(max_time)
//                    [0, log10(2), log10(5)] = log10(max_time) - x - 1
//  Let s = log10(max_time) - 1
//  Then we have
//   [0, 0.3, 0.7] + x = s, where x is an integer
//  If we take x = floor(s)
//  Then we have
//   [0, 0.3, 0.7] = s - floor(s)
//   We choose the closest one.

impl DisplayMetrics {
    fn compute_major_tick_delta_t(&self) -> u64 {
        let delta_t = (self.max_time - self.min_time) as f64;
        let s = delta_t.log10() - 1.0;
        let x = s.floor();
        let e = s - x;
        let d0 = (e - 0.0).abs();
        let d1 = (e - 2.0_f64.log10()).abs();
        let d2 = (e - 5.0_f64.log10()).abs();
        let value = if d0 <= d1 && d0 <= d2 {
            (10.0_f64.powf(x)) as u64
        } else if d1 <= d0 && d1 <= d2 {
            (2.0_f64 * 10.0_f64.powf(x)) as u64
        } else {
            (5.0_f64 * 10.0_f64.powf(x)) as u64
        };
        value
    }

    fn time_to_pixel(&self, time: u64) -> Option<u32> {
        if time < self.min_time || time > self.max_time {
            None
        } else {
            Some(
                (self.signal_width as f64 + self.pixel_scale() * ((time - self.min_time) as f64))
                    .round() as u32,
            )
        }
    }

    fn major_tick_distance(&self) -> u32 {
        (self.compute_major_tick_delta_t() as f64 * self.pixel_scale()).round() as u32
    }

    fn minor_tick_distance(&self) -> u32 {
        self.major_tick_distance() / 5
    }

    fn pixel_scale(&self) -> f64 {
        ((self.canvas_width - self.signal_width + 1) as f64)
            / ((self.max_time - self.min_time) as f64)
    }

    fn time_view(&self) -> TimeView {
        TimeView {
            start_time: self.min_time,
            end_time: self.max_time,
            pixel_scale: self.pixel_scale(),
        }
    }

    pub fn major_x0(&self, major: u64) -> Option<u32> {
        let value = self.compute_major_tick_delta_t() * major;
        self.time_to_pixel(value)
    }

    pub fn minor_x0(&self, major: u64, minor: u32) -> Option<u32> {
        if let Some(major) = self.major_x0(major) {
            Some(major + (minor + 1) * self.minor_tick_distance())
        } else {
            None
        }
    }
    pub fn signal_baseline(&self, index: usize) -> u32 {
        self.timescale_height + ((index as u32 + 1) * self.signal_height)
    }
    fn signal_rect(&self) -> Path {
        rect(0, 0, self.signal_width, self.canvas_height, "#e8e8e8")
    }

    fn background_rect(&self) -> Path {
        rect(0, 0, self.canvas_width, self.canvas_height, "#282828")
    }

    fn timescale_header_rect(&self) -> Path {
        rect(
            self.signal_width,
            0,
            self.canvas_width,
            self.timescale_height,
            "#f3f5de",
        )
    }

    fn timescale_midline(&self) -> Path {
        line(
            self.signal_width,
            self.timescale_midline,
            self.canvas_width,
            self.timescale_midline,
            "#cbcbcb",
        )
    }

    fn timescale_major_tick(&self, major: u64) -> Option<Path> {
        if let Some(x0) = self.major_x0(major) {
            Some(line(
                x0,
                self.timescale_midline - self.tick_half_height,
                x0,
                self.timescale_midline + self.tick_half_height,
                "#000000",
            ))
        } else {
            None
        }
    }

    fn timescale_minor_tick(&self, major: u64, minor: u32) -> Option<Path> {
        if let Some(x1) = self.minor_x0(major, minor) {
            Some(line(
                x1,
                self.timescale_midline,
                x1,
                self.timescale_midline + self.tick_half_height,
                "#000000",
            ))
        } else {
            None
        }
    }

    fn timescale_major_label(&self, major: u64, value: &str) -> Option<Text> {
        if let Some(x0) = self.major_x0(major) {
            let txt = Text::new()
                .add(svg::node::Text::new(value))
                .set("x", x0)
                .set(
                    "y",
                    self.timescale_midline + self.tick_half_height + self.shim,
                )
                .set("text-anchor", "middle")
                .set("font-family", "sans-serif")
                .set("alignment-baseline", "hanging")
                .set("font-size", self.label_size);
            Some(txt)
        } else {
            None
        }
    }

    fn signal_label(&self, index: usize, signal: &str) -> Text {
        Text::new()
            .add(svg::node::Text::new(signal))
            .set("x", self.shim)
            .set("y", self.signal_baseline(index) - self.shim)
            .set("text-anchor", "start")
            .set("font-family", "sans-serif")
            .set("alignment-baseline", "bottom")
            .set("font-size", self.label_size)
    }

    fn signal_line(&self, index: usize) -> Path {
        let y0 = self.signal_baseline(index);
        line(0, y0, self.signal_width, y0, "#cbcbcb")
    }

    fn timescale(&self, mut document: SVG) -> SVG {
        let first_major_tick =
            (self.min_time as f64 / self.compute_major_tick_delta_t() as f64).floor() as u64;
        let last_major_tick =
            (self.max_time as f64 / self.compute_major_tick_delta_t() as f64).ceil() as u64;
        let delt = self.compute_major_tick_delta_t();
        for major in first_major_tick..=last_major_tick {
            if let Some(major_tick) = self.timescale_major_tick(major) {
                document = document.add(major_tick);
            }
            if let Some(label) = self.timescale_major_label(major, &format!("{}ps", delt * major)) {
                document = document.add(label);
            }
            for minor in 0..4 {
                if let Some(minor_tick) = self.timescale_minor_tick(major, minor) {
                    document = document.add(minor_tick);
                }
            }
        }
        document
    }

    fn vector_signal_plot(&self, index: usize, values: &[TimedValue<BigInt>], mut doc: SVG) -> SVG {
        let values = changes(values);
        let time_view = self.time_view();
        let y0 = self.signal_baseline(index);
        let y_lo = (y0 - self.signal_height + self.shim) as f64;
        let y_hi = (y0 - self.shim) as f64;
        let flip = |x| if x == y_lo { y_hi } else { y_lo };
        let shim = (0.5 * time_view.pixel_scale).min(1.0);
        let x0 = self.signal_width as f64;
        let mut data = Data::new().move_to((x0, y_lo));
        let mut data_reverse = Data::new().move_to((x0, y_hi));
        let mut last_y1 = y_lo as f64;
        for value in time_view
            .intervals(&values)
            .iter()
            .filter(|x| !x.is_empty())
        {
            let x1 = x0 + value.start_x;
            let y1 = flip(last_y1);
            data = data.line_to((x1 - shim, last_y1));
            data_reverse = data_reverse.line_to((x1 - shim, flip(last_y1)));
            last_y1 = y1;
            data = data.line_to((x1 + shim, y1));
            data_reverse = data_reverse.line_to((x1 + shim, flip(y1)));
            doc = doc.add(
                Text::new()
                    .add(svg::node::Text::new(value.label.to_string()))
                    .set("x", x1 + 2.0 * shim)
                    .set(
                        "y",
                        self.signal_baseline(index) - self.signal_height / 2 + 1,
                    )
                    .set("text-anchor", "start")
                    .set("font-family", "sans-serif")
                    .set("alignment-baseline", "middle")
                    .set("font-size", self.label_size - 2)
                    .set("fill", "white"),
            );
        }
        let doc = doc
            .add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", "#87ecd1")
                    .set("stroke-width", 0.75)
                    .set("d", data),
            )
            .add(
                Path::new()
                    .set("fill", "none")
                    .set("stroke", "#87ecd1")
                    .set("stroke-width", 0.75)
                    .set("d", data_reverse),
            );
        doc
    }
    fn bit_signal_plot(&self, index: usize, values: &[TimedValue<bool>]) -> Path {
        let values = changes(values);
        let y0 = self.signal_baseline(index) - self.shim;
        let y1 = y0 - self.signal_height + self.shim * 2;
        let x0 = self.signal_width;
        let mut data = Data::new().move_to((x0, y0));
        let mut last_y1 = y0;
        for value in values {
            if let Some(x1) = self.time_to_pixel(value.time) {
                let y = if value.value { y1 } else { y0 };
                data = data.line_to((x1, last_y1));
                last_y1 = y;
                data = data.line_to((x1, y));
            }
        }
        if let Some(x) = self.time_to_pixel(self.max_time) {
            data = data.line_to((x, last_y1));
        }
        Path::new()
            .set("fill", "none")
            .set("stroke", "#87ecd1")
            .set("stroke-width", 0.75)
            .set("d", data)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signals = ["time.clock.d", "foo.bar.input.size.q", "blah.blah.d.next"];

    let metrics = DisplayMetrics::default();

    let mut document = Document::new()
        .set(
            "viewBox",
            (0, 0, metrics.canvas_width, metrics.canvas_height),
        )
        .add(metrics.background_rect());

    // Paint the timescale rectangle
    let mut document = document
        .add(metrics.signal_rect())
        .add(metrics.timescale_header_rect())
        .add(metrics.timescale_midline());

    document = metrics.timescale(document);

    for (index, signal) in signals.iter().cycle().take(16).enumerate() {
        document = document
            .add(metrics.signal_label(index, signal))
            .add(metrics.signal_line(index))
            .add(metrics.bit_signal_plot(index, &make_clock(1 + index as u64))) //)&make_clock(5*(1+index as u64))))
    }
    document = document
        .add(metrics.signal_label(16, "counter"))
        .add(metrics.signal_line(16));
    document = metrics.vector_signal_plot(16, &make_linear_counter(10), document);
    svg::save("image.svg", &document).unwrap();

    Ok(())
}
