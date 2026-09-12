use super::types::{CellEntry, TableGrid};
use crate::parser::types::StyledToken;
use crate::parser::utils::{
    decode_html_entities, extract_title_from_href, to_subscript_str, to_superscript_str,
};
use crate::theme;
use ratatui::style::{Modifier, Style};

pub fn parse_table_into_grid<'a>(
    table_tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    show_footnotes: bool,
) -> Option<TableGrid> {
    let mut caption = None;
    let mut raw_tr_elements = Vec::new();

    for handle in table_tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = handle.get(parser) {
            match child_tag.name().as_utf8_str().as_ref() {
                "caption" => {
                    let text = child_tag.inner_text(parser);
                    let clean = decode_html_entities(&text).trim().to_string();
                    if !clean.is_empty() {
                        caption = Some(clean);
                    }
                }
                "tr" => raw_tr_elements.push(child_tag),
                "thead" | "tbody" | "tfoot" => {
                    for sub_handle in child_tag.children().top().iter() {
                        if let Some(tl::Node::Tag(sub_tag)) = sub_handle.get(parser)
                            && sub_tag.name().as_utf8_str() == "tr" {
                                raw_tr_elements.push(sub_tag);
                            }
                    }
                }
                _ => {}
            }
        }
    }

    if raw_tr_elements.is_empty() {
        return None;
    }

    let mut grid: Vec<Vec<Option<CellEntry>>> = Vec::new();

    for (r, tr_tag) in raw_tr_elements.iter().enumerate() {
        if grid.len() <= r {
            grid.resize(r + 1, Vec::new());
        }
        let mut c = 0;

        for cell_handle in tr_tag.children().top().iter() {
            if let Some(tl::Node::Tag(cell_tag)) = cell_handle.get(parser) {
                let name = cell_tag.name().as_utf8_str();
                let name_str = name.as_ref();
                if name_str == "th" || name_str == "td" {
                    let is_header = name_str == "th";
                    let colspan = cell_tag
                        .attributes()
                        .get("colspan")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse().ok())
                        .unwrap_or(1)
                        .max(1);
                    let rowspan = cell_tag
                        .attributes()
                        .get("rowspan")
                        .flatten()
                        .and_then(|v| v.as_utf8_str().parse().ok())
                        .unwrap_or(1)
                        .max(1);

                    let default_style = if is_header {
                        Style::default()
                            .fg(theme::BEIGE)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme::FG)
                    };

                    let mut tokens = Vec::new();
                    collect_cell_tokens(
                        cell_tag,
                        parser,
                        default_style,
                        None,
                        &mut tokens,
                        show_footnotes,
                        false,
                        false,
                    );

                    while c < grid[r].len() && grid[r][c].is_some() {
                        c += 1;
                    }

                    if grid.len() < r + rowspan {
                        grid.resize(r + rowspan, Vec::new());
                    }
                    for row_idx in r..(r + rowspan) {
                        if grid[row_idx].len() < c + colspan {
                            grid[row_idx].resize(c + colspan, None);
                        }
                    }

                    grid[r][c] = Some(CellEntry::Origin {
                        tokens,
                        colspan,
                        rowspan,
                        is_header,
                    });

                    for row_offset in 0..rowspan {
                        for col_offset in 0..colspan {
                            if row_offset == 0 && col_offset == 0 {
                                continue;
                            }
                            grid[r + row_offset][c + col_offset] = Some(CellEntry::Covered {
                                origin_r: r,
                                origin_c: c,
                            });
                        }
                    }

                    c += colspan;
                }
            }
        }
    }

    let num_rows = grid.len();
    let num_cols = grid.iter().map(|row| row.len()).max().unwrap_or(0);

    if num_rows == 0 || num_cols == 0 {
        return None;
    }

    let clean_grid: Vec<Vec<CellEntry>> = grid
        .into_iter()
        .map(|mut row| {
            row.resize(num_cols, None);
            row.into_iter()
                .map(|opt| {
                    opt.unwrap_or_else(|| CellEntry::Origin {
                        tokens: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        is_header: false,
                    })
                })
                .collect()
        })
        .collect();

    Some(TableGrid {
        num_rows,
        num_cols,
        cells: clean_grid,
        caption,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_cell_tokens<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    style: Style,
    link: Option<String>,
    tokens: &mut Vec<StyledToken>,
    show_footnotes: bool,
    is_sup: bool,
    is_sub: bool,
) {
    let tag_name = tag.name().as_utf8_str();
    let tag_name_str = tag_name.as_ref();
    if matches!(tag_name_str, "style" | "script" | "noscript" | "annotation") {
        return;
    }
    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
        && !show_footnotes && cls.contains("reference") {
            return;
        }

    let mut current_style = style;
    let mut current_link = link;
    let current_is_sup = is_sup || tag_name_str == "sup";
    let current_is_sub = is_sub || tag_name_str == "sub";

    match tag_name_str {
        "b" | "strong" | "th" => current_style = current_style.add_modifier(Modifier::BOLD),
        "i" | "em" => current_style = current_style.add_modifier(Modifier::ITALIC),
        "a" => {
            current_style = current_style.fg(theme::BLUE);
            if let Some(href) = tag
                .attributes()
                .get("href")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                if let Some(title) = extract_title_from_href(href.as_ref()) {
                    current_link = Some(title);
                }
            } else if let Some(title_attr) = tag
                .attributes()
                .get("title")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                current_link = Some(title_attr.to_string());
            }
        }
        "code" | "kbd" | "samp" | "tt" => current_style = current_style.fg(theme::ORANGE),
        "abbr" => current_style = current_style.add_modifier(Modifier::UNDERLINED),
        "br" => tokens.push(StyledToken {
            text: "\n".to_string(),
            style: current_style,
            link_target: None,
        }),
        _ => {}
    }

    for child_handle in tag.children().top().iter() {
        if let Some(child_node) = child_handle.get(parser) {
            match child_node {
                tl::Node::Raw(bytes) => {
                    let raw_text = bytes.as_utf8_str();
                    let decoded_text = decode_html_entities(&raw_text);
                    let transformed = if current_is_sup {
                        to_superscript_str(&decoded_text)
                    } else if current_is_sub {
                        to_subscript_str(&decoded_text)
                    } else {
                        decoded_text.into_owned()
                    };
                    let cleaned = transformed.replace(['\r', '\t', '\n'], " ");
                    if !cleaned.trim().is_empty() || cleaned == " " {
                        tokens.push(StyledToken {
                            text: cleaned,
                            style: current_style,
                            link_target: current_link.clone(),
                        });
                    }
                }
                tl::Node::Tag(child_tag) => {
                    collect_cell_tokens(
                        child_tag,
                        parser,
                        current_style,
                        current_link.clone(),
                        tokens,
                        show_footnotes,
                        current_is_sup,
                        current_is_sub,
                    );
                }
                _ => {}
            }
        }
    }
}
