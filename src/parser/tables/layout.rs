use super::types::{CellEntry, TableGrid};
use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

pub fn compute_column_widths(grid: &TableGrid, max_width: usize) -> Vec<usize> {
    let num_rows = grid.num_rows;
    let num_cols = grid.num_cols;

    let mut min_widths = vec![3usize; num_cols];
    let mut max_widths = vec![3usize; num_cols];

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin {
                tokens, colspan, ..
            } = &grid.cells[r][c]
            {
                let full_text: String = tokens.iter().map(|t| t.text.as_str()).collect();
                let longest_word = full_text
                    .split_whitespace()
                    .map(UnicodeWidthStr::width)
                    .max()
                    .unwrap_or(0);
                let total_width = UnicodeWidthStr::width(full_text.trim());

                if *colspan == 1 {
                    min_widths[c] = min_widths[c].max(longest_word.max(3));
                    max_widths[c] = max_widths[c].max(total_width.max(3));
                } else {
                    let share_min = (longest_word / *colspan).max(3);
                    let share_max = (total_width / *colspan).max(3);
                    for dc in 0..*colspan {
                        if c + dc < num_cols {
                            min_widths[c + dc] = min_widths[c + dc].max(share_min);
                            max_widths[c + dc] = max_widths[c + dc].max(share_max);
                        }
                    }
                }
            }
        }
    }

    let overhead = 3 * num_cols + 1;
    let available_width = max_width.saturating_sub(overhead).max(num_cols * 3);
    let total_max: usize = max_widths.iter().sum();
    let mut col_widths = vec![3usize; num_cols];

    if total_max <= available_width {
        for (i, w) in max_widths.iter().enumerate() {
            col_widths[i] = (*w).max(3);
        }
    } else {
        for i in 0..num_cols {
            let prop =
                (max_widths[i] as f64 / total_max as f64 * available_width as f64).round() as usize;
            col_widths[i] = prop.max(min_widths[i].min(15)).max(3);
        }
        let total_alloc: usize = col_widths.iter().sum();
        if total_alloc > available_width {
            let mut diff = total_alloc - available_width;
            for i in (0..num_cols).rev() {
                if col_widths[i] > 3 {
                    let shrink = (col_widths[i] - 3).min(diff);
                    col_widths[i] -= shrink;
                    diff -= shrink;
                    if diff == 0 {
                        break;
                    }
                }
            }
        }
    }

    col_widths
}

pub fn compute_row_heights(
    grid: &TableGrid,
    origin_lines: &[Vec<Vec<Span<'static>>>],
) -> Vec<usize> {
    let num_rows = grid.num_rows;
    let num_cols = grid.num_cols;

    let mut row_heights = vec![1usize; num_rows];
    for r in 0..num_rows {
        let mut max_h = 1;
        for c in 0..num_cols {
            if let CellEntry::Origin { rowspan, .. } = &grid.cells[r][c]
                && *rowspan == 1 {
                    max_h = max_h.max(origin_lines[r * num_cols + c].len());
                }
        }
        row_heights[r] = max_h;
    }

    for r in 0..num_rows {
        for c in 0..num_cols {
            if let CellEntry::Origin { rowspan, .. } = &grid.cells[r][c]
                && *rowspan > 1 {
                    let needed = origin_lines[r * num_cols + c].len();
                    let current_total: usize =
                        (0..*rowspan).filter_map(|dr| row_heights.get(r + dr)).sum();
                    if needed > current_total {
                        let end_row = (r + *rowspan - 1).min(num_rows - 1);
                        row_heights[end_row] += needed - current_total;
                    }
                }
        }
    }

    row_heights
}
