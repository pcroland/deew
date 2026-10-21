//! Progress display, modelled on `rich.progress.Progress` with the columns
//! `[`, description, `]`, bar and a magenta percentage.

use crate::markup::{color_enabled, render, visible_width};
use crate::term::console_width;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::sync::Mutex;
use std::time::Duration;

/// Width of the bar when there is enough room.
const BAR_WIDTH: usize = 40;
/// Width of the percentage column (`100.00%`).
const PERCENT_WIDTH: usize = 7;
/// Positions are hundredths of a percent.
const SCALE: u64 = 10_000;

// 256-color approximations of rich's bar.complete, bar.finished and bar.back.
const COMPLETE: &str = "\x1b[38;5;197m";
const FINISHED: &str = "\x1b[38;5;70m";
const BACK: &str = "\x1b[38;5;59m";
const RESET: &str = "\x1b[0m";

/// Width of the description column in the running state: every description
/// has this many characters (rich aligns the column across tasks).
pub const DESCRIPTION_WIDTH: usize = 43;

/// Number of pulse positions per second, like rich.
const PULSE_SPEED: f64 = 15.0;
const PULSE_SIZE: usize = 20;

/// Colour of the pulse at `index` of its cycle: pink fading to grey.
fn pulse_color(index: usize) -> &'static str {
    let angle = 2.0 * std::f64::consts::PI * index as f64 / PULSE_SIZE as f64;
    let ratio = (1.0 + angle.cos()) / 2.0;
    if ratio > 0.75 {
        COMPLETE
    } else if ratio > 0.4 {
        "\x1b[38;5;132m"
    } else if ratio > 0.15 {
        "\x1b[38;5;96m"
    } else {
        BACK
    }
}

/// Width of the bar: 40 cells, less if the line would not fit the console
/// (`[ `, description, ` ] `, bar, a space and the percentage).
pub fn bar_width(console: usize, description: usize) -> usize {
    let fixed = 2 + description + 3 + 1 + PERCENT_WIDTH;
    BAR_WIDTH.min(console.saturating_sub(fixed)).max(1)
}

/// The bar, `width` cells wide. `fraction` is `None` for an unknown total
/// (a pulsing bar, `time` in seconds moves it).
pub fn bar_text(fraction: Option<f64>, color: bool, time: f64, width: usize) -> String {
    let mut out = String::new();
    match fraction {
        Some(fraction) => {
            let fraction = fraction.clamp(0.0, 1.0);
            let halves = (width as f64 * 2.0 * fraction) as usize;
            let (full, half) = (halves / 2, halves % 2);
            let finished = fraction >= 1.0;
            if color {
                out.push_str(if finished { FINISHED } else { COMPLETE });
            }
            out.push_str(&"━".repeat(full));
            out.push_str(&"╸".repeat(half));
            let mut remaining = width - full - half;
            if color && remaining > 0 {
                out.push_str(BACK);
                if half == 0 && full > 0 {
                    out.push('╺');
                    remaining -= 1;
                }
            }
            if color {
                out.push_str(&"━".repeat(remaining));
                out.push_str(RESET);
            } else {
                out.push_str(&" ".repeat(remaining));
            }
        }
        None => {
            let offset = (-(time * PULSE_SPEED)).rem_euclid(PULSE_SIZE as f64) as usize;
            for i in 0..width {
                let index = (i + offset) % PULSE_SIZE;
                if color {
                    out.push_str(pulse_color(index));
                }
                out.push('━');
            }
            if color {
                out.push_str(RESET);
            }
        }
    }
    out
}

/// `{task.percentage:>3.2f}%` in magenta, padded to the column width.
pub fn percent_text(fraction: Option<f64>, color: bool) -> String {
    let text = format!(
        "{:<PERCENT_WIDTH$}",
        format!("{:.2}%", fraction.unwrap_or(0.0) * 100.0)
    );
    if color {
        format!("\x1b[35m{text}{RESET}")
    } else {
        text
    }
}

fn style() -> ProgressStyle {
    let color = color_enabled();
    let fraction = |s: &ProgressState| s.len().map(|len| s.pos() as f64 / len as f64);
    ProgressStyle::with_template("[ {msg} ] {deewbar} {deewpct}")
        .expect("valid template")
        .with_key("deewbar", move |s: &ProgressState, w: &mut dyn Write| {
            let width = bar_width(console_width(), DESCRIPTION_WIDTH);
            let _ = w.write_str(&bar_text(
                fraction(s),
                color,
                s.elapsed().as_secs_f64(),
                width,
            ));
        })
        .with_key("deewpct", move |s: &ProgressState, w: &mut dyn Write| {
            let _ = w.write_str(&percent_text(fraction(s), color));
        })
}

/// A progress line of one job.
pub struct Bar {
    pb: ProgressBar,
    /// Last description (markup) and fraction, for the output without a
    /// terminal.
    last: Mutex<(String, Option<f64>)>,
}

impl Bar {
    pub fn describe(&self, markup: &str) {
        self.pb.set_message(render(markup, color_enabled()));
        self.last.lock().expect("bar state").0 = markup.to_string();
    }

    /// Set the progress, 0 to 100.
    pub fn percent(&self, pct: f64) {
        let pct = if pct.is_nan() {
            0.0
        } else {
            pct.clamp(0.0, 100.0)
        };
        if self.pb.length().is_none() {
            self.pb.set_length(SCALE);
        }
        self.pb.set_position((pct * 100.0).round() as u64);
        self.last.lock().expect("bar state").1 = Some(pct / 100.0);
    }

    /// Unknown total: show a pulsing bar.
    pub fn indeterminate(&self) {
        self.pb.unset_length();
        self.last.lock().expect("bar state").1 = None;
    }
}

/// All progress lines plus a way to print messages above them.
pub struct Ui {
    multi: MultiProgress,
    finals: Mutex<Vec<(usize, String, Option<f64>)>>,
}

impl Ui {
    pub fn new() -> Ui {
        Ui {
            multi: MultiProgress::with_draw_target(ProgressDrawTarget::stdout_with_hz(8)),
            finals: Mutex::new(Vec::new()),
        }
    }

    /// Add a progress line (at the bottom).
    pub fn add_bar(&self) -> Bar {
        let pb = self.multi.add(ProgressBar::new(SCALE));
        pb.set_style(style());
        pb.enable_steady_tick(Duration::from_millis(125));
        Bar {
            pb,
            last: Mutex::new((String::new(), Some(0.0))),
        }
    }

    /// The job is done: keep its line as it is.
    pub fn finish(&self, index: usize, bar: Bar) {
        bar.pb.disable_steady_tick();
        bar.pb.abandon();
        let (description, fraction) = bar.last.lock().expect("bar state").clone();
        self.finals
            .lock()
            .expect("finals")
            .push((index, description, fraction));
    }

    /// Print a line above the progress lines. `markup` is rendered, plain
    /// text has to be escaped by the caller.
    pub fn say(&self, markup: &str) {
        let text = render(markup, color_enabled());
        self.say_raw(&text);
    }

    /// Print text that is not parsed as markup.
    pub fn say_raw(&self, text: &str) {
        if self.multi.is_hidden() {
            println!("{text}");
        } else {
            let _ = self.multi.println(text);
        }
    }

    /// Without a terminal nothing was drawn: print the final state of every
    /// line, like rich does when the output is not a terminal.
    pub fn flush(self) {
        if !self.multi.is_hidden() {
            return;
        }
        let mut finals = self.finals.into_inner().expect("finals");
        finals.sort_by_key(|(index, _, _)| *index);
        let descriptions: Vec<String> = finals.iter().map(|(_, d, _)| render(d, false)).collect();
        let width = descriptions
            .iter()
            .map(|d| visible_width(d))
            .max()
            .unwrap_or(0);
        let percents: Vec<String> = finals
            .iter()
            .map(|(_, _, f)| percent_text(*f, false))
            .collect();
        let bar_width = bar_width(console_width(), width);
        for ((description, (_, _, fraction)), percent) in
            descriptions.iter().zip(&finals).zip(&percents)
        {
            let fill = " ".repeat(width.saturating_sub(visible_width(description)));
            println!(
                "[ {description}{fill} ] {} {percent}",
                bar_text(*fraction, false, 0.0, bar_width)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::strip;

    #[test]
    fn bar_without_color() {
        assert_eq!(bar_text(Some(1.0), false, 0.0, 40), "━".repeat(40));
        assert_eq!(bar_text(Some(0.0), false, 0.0, 40), " ".repeat(40));
        // 45.5%: 18 full cells (36 halves of 36.4)
        assert_eq!(
            bar_text(Some(0.455), false, 0.0, 40),
            format!("{}{}", "━".repeat(18), " ".repeat(22))
        );
        assert_eq!(
            bar_text(Some(0.5), false, 0.0, 40)
                .chars()
                .filter(|&c| c == '━')
                .count(),
            20
        );
        // a half cell
        assert_eq!(
            bar_text(Some(0.4625), false, 0.0, 40),
            format!("{}╸{}", "━".repeat(18), " ".repeat(21))
        );
        for f in [0.0, 0.3, 0.999, 1.0, 7.0, -1.0] {
            assert_eq!(
                strip(&bar_text(Some(f), false, 0.0, 40)).chars().count(),
                40
            );
            assert!(strip(&bar_text(Some(f), true, 0.0, 40)).chars().count() >= 40);
        }
    }

    #[test]
    fn bar_shrinks_to_fit_the_console() {
        assert_eq!(bar_width(250, DESCRIPTION_WIDTH), 40);
        assert_eq!(bar_width(96, DESCRIPTION_WIDTH), 40);
        assert_eq!(bar_width(95, DESCRIPTION_WIDTH), 39);
        // what rich does at its default width of 80
        assert_eq!(bar_width(80, DESCRIPTION_WIDTH), 24);
        assert_eq!(bar_width(10, DESCRIPTION_WIDTH), 1);
        assert_eq!(bar_text(Some(1.0), false, 0.0, 24), "━".repeat(24));
        assert_eq!(bar_text(None, false, 0.0, 7).chars().count(), 7);
    }

    #[test]
    fn pulsing_bar_is_full_width_and_moves() {
        let a = bar_text(None, true, 0.0, 40);
        let b = bar_text(None, true, 0.2, 40);
        assert_ne!(a, b);
        assert_eq!(bar_text(None, false, 3.0, 40), "━".repeat(40));
    }

    #[test]
    fn colored_bar_uses_back_style_for_the_rest() {
        let text = bar_text(Some(0.5), true, 0.0, 40);
        assert!(text.starts_with(COMPLETE));
        assert!(text.contains(BACK));
        assert!(text.contains("╺"));
        assert!(bar_text(Some(1.0), true, 0.0, 40).starts_with(FINISHED));
    }

    #[test]
    fn percentage() {
        assert_eq!(percent_text(Some(0.4555), false), "45.55% ");
        assert_eq!(percent_text(Some(1.0), false), "100.00%");
        assert_eq!(percent_text(None, false), "0.00%  ");
        assert_eq!(percent_text(Some(0.0), true), "\x1b[35m0.00%  \x1b[0m");
    }
}
