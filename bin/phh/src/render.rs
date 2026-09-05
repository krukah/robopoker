//! Shared table formatting. Plain fixed-width text — these reports are read in
//! a terminal and diffed between blueprint versions, so stable column widths
//! matter more than prettiness.

/// A fixed-width table with a header rule.
pub struct Table {
    widths: Vec<usize>,
    rows: Vec<Vec<String>>,
}

impl Table {
    /// `header` doubles as the column-width floor.
    pub fn new(header: &[&str], widths: &[usize]) -> Self {
        Self {
            widths: widths.to_vec(),
            rows: vec![header.iter().map(|h| (*h).to_string()).collect()],
        }
    }

    pub fn push(&mut self, cells: Vec<String>) {
        self.rows.push(cells);
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (index, row) in self.rows.iter().enumerate() {
            let line = row
                .iter()
                .zip(self.widths.iter())
                .map(|(cell, width)| match width {
                    0 => cell.clone(),
                    w => format!("{cell:>w$}"),
                })
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(f, "{}", line.trim_end())?;
            if index == 0 {
                writeln!(f, "{}", "-".repeat(self.widths.iter().sum::<usize>() + self.widths.len() - 1))?;
            }
        }
        Ok(())
    }
}

/// `1234` → `1,234`, so six-figure counts stay readable.
pub fn commas(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// A share of a whole, as a percentage with one decimal.
pub fn pct(part: usize, whole: usize) -> String {
    match whole {
        0 => "—".to_string(),
        w => format!("{:.1}%", 100.0 * part as f64 / w as f64),
    }
}
