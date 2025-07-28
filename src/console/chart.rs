// use crate::IO;

#[derive(Debug)]
pub struct BarValue {
    value: f64,
    prefix: String,
    suffix: String,
}

pub struct BarChart {
    draw_char: char,
    max_bar_length: usize,
    data: Vec<BarValue>,
}

impl BarChart {
    pub fn new(draw_char: char, max_bar_length: usize) -> Self {
        BarChart {
            draw_char,
            max_bar_length,
            data: Vec::new(),
        }
    }

    pub fn default() -> Self {
        Self::new('▇', 30)
    }

    pub fn add_value(&mut self, value: f64, prefix: &str, suffix: &str) {
        self.data.push(BarValue {
            value,
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        });
    }

    pub fn get(&mut self, reverse: bool) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        self.data.sort_by(|a, b| {
            if reverse {
                b.value.partial_cmp(&a.value)
            } else {
                a.value.partial_cmp(&b.value)
            }
            .unwrap_or(std::cmp::Ordering::Equal)
        });

        let max_value = if reverse {
            self.data[0].value
        } else {
            self.data.last().unwrap().value
        };

        let max_prefix_length = self
            .data
            .iter()
            .map(|d| d.prefix.len())
            .max()
            .unwrap_or(0)
            + 1;

        let mut chart = String::new();

        for item in &self.data {
            let bar_length = if max_value == 0.0 {
                0
            } else {
                Self::remap(item.value, 0.0, max_value, 0.0, self.max_bar_length as f64).round() as usize
            };

            let line = format!(
                "{}{}: {} {}\n",
                item.prefix,
                " ".repeat(max_prefix_length - item.prefix.len()),
                self.draw_char.to_string().repeat(bar_length),
                item.suffix
            );
            chart.push_str(&line);
        }

        if chart.ends_with('\n') {
            chart.pop();
        }

        chart
    }

    fn remap(n: f64, old_min: f64, old_max: f64, new_min: f64, new_max: f64) -> f64 {
        ((n - old_min) * (new_max - new_min)) / (old_max - old_min) + new_min
    }
}
