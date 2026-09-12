use crate::feed::FeedState;
use crate::theme;
use crate::ui::modals::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, Clear, Paragraph, Wrap},
    Frame,
};

pub fn compute_feed_card_area(inner_area: Rect) -> Rect {
    centered_rect(80, 85, inner_area)
}

pub fn render_feed_view(f: &mut Frame, feed: &FeedState, area: Rect, rounded: bool) {
    f.render_widget(Clear, area);

    let border_type = theme::border_type(rounded);

    let main_block = Block::bordered()
        .border_type(border_type)
        .border_style(Style::default().fg(theme::VIOLET))
        .title(Title::from(" wikipedia feed ").alignment(Alignment::Center));

    f.render_widget(main_block.clone(), area);
    let inner_area = main_block.inner(area);

    if feed.items.is_empty() {
        let vertical_offset = (inner_area.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }

        let spinner = crate::ui::current_spinner_frame();

        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::LIME).bold(),
            ),
            Span::styled(
                "fetching articles for your feed...",
                Style::default().fg(theme::BEIGE).bold(),
            ),
        ]));

        let loading_p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(loading_p, inner_area);
        return;
    }

    let active_idx = feed.active_idx;
    let item = &feed.items[active_idx];

    let card_area = centered_rect(80, 85, inner_area);
    f.render_widget(Clear, card_area);

    let card_border_color = if item.is_liked {
        theme::LIME
    } else {
        theme::PINK
    };

    let like_badge = if item.is_liked { " liked " } else { "" };
    let like_style = if item.is_liked {
        Style::default().fg(theme::LIME).bold()
    } else {
        Style::default().fg(theme::GREY)
    };

    let post_title = if feed.is_fetching {
        format!(
            " post {} of {} · {} fetching more... ",
            active_idx + 1,
            feed.items.len(),
            crate::ui::current_spinner_frame()
        )
    } else {
        format!(" post {} of {} ", active_idx + 1, feed.items.len())
    };

    let card_block = Block::bordered()
        .border_type(border_type)
        .border_style(Style::default().fg(card_border_color))
        .title(Title::from(format!(" {} ", item.title.to_lowercase())).alignment(Alignment::Center))
        .title(
            Title::from(post_title)
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Left),
        )
        .title(
            Title::from(Span::styled(like_badge, like_style))
                .position(ratatui::widgets::block::Position::Bottom)
                .alignment(Alignment::Right),
        );

    let mut card_lines = Vec::new();
    card_lines.push(Line::from(""));

    if let Some(short_desc) = &item.short_description
        && !short_desc.is_empty() {
            card_lines.push(
                Line::from(Span::styled(
                    short_desc.clone(),
                    Style::default().fg(theme::GREY).italic(),
                ))
                .alignment(Alignment::Center),
            );
            card_lines.push(Line::from(""));
        }

    if !item.snippet.is_empty() {
        card_lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(&item.snippet, Style::default().fg(theme::FG)),
        ]));
    } else {
        card_lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                "press enter to read full article...",
                Style::default().fg(theme::GREY).italic(),
            ),
        ]));
    }

    card_lines.push(Line::from(""));
    if !item.categories.is_empty() {
        let max_display = 3;
        let mut spans = vec![Span::styled(
            "   categories: ",
            Style::default().fg(theme::GREY),
        )];

        let mut sorted_indices: Vec<usize> = (0..item.categories.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let score_b = feed
                .profile
                .score_for_categories(std::slice::from_ref(&item.categories[b]));
            let score_a = feed
                .profile
                .score_for_categories(std::slice::from_ref(&item.categories[a]));
            score_b.cmp(&score_a)
        });

        for (i, &idx) in sorted_indices.iter().take(max_display).enumerate() {
            if i > 0 {
                spans.push(Span::styled(" · ", Style::default().fg(theme::GREY)));
            }
            let cat = &item.categories[idx];
            let is_matched = feed.profile.score_for_categories(std::slice::from_ref(cat)) > 0;
            let cat_color = if is_matched {
                theme::VIOLET
            } else {
                theme::BEIGE
            };
            spans.push(Span::styled(cat.as_str(), Style::default().fg(cat_color)));
        }

        if item.categories.len() > max_display {
            let remaining = item.categories.len() - max_display;
            spans.push(Span::styled(
                format!(" (+{} more)", remaining),
                Style::default().fg(theme::GREY).italic(),
            ));
        }

        card_lines.push(Line::from(spans));
    }

    let card_p = Paragraph::new(card_lines)
        .block(card_block)
        .wrap(Wrap { trim: true });
    f.render_widget(card_p, card_area);
}
