use ratatui::{
    buffer::Buffer,
    layout::{Direction, Rect},
    prelude::BlockExt,
    style::{Style, Styled},
    symbols,
    text::Line,
    widgets::{Block, Widget, WidgetRef},
};

mod bar;
mod bar_group;

pub use bar::Bar;
pub use bar_group::BarGroup;

/// A chart showing values as [bars](Bar).
///
/// Here is a possible `BarChart` output.
/// ```plain
/// ┌─────────────────────────────────┐
/// │                             ████│
/// │                        ▅▅▅▅ ████│
/// │            ▇▇▇▇        ████ ████│
/// │     ▄▄▄▄   ████ ████   ████ ████│
/// │▆10▆ █20█   █50█ █40█   █60█ █90█│
/// │ B1   B2     B1   B2     B1   B2 │
/// │ Group1      Group2      Group3  │
/// └─────────────────────────────────┘
/// ```
///
/// A `BarChart` is composed of a set of [`Bar`] which can be set via [`BarChart::data`].
/// Bars can be styled globally ([`BarChart::bar_style`]) or individually ([`Bar::style`]).
/// There are other methods available to style even more precisely. See [`Bar`] to find out about
/// each bar component.
///
/// The `BarChart` widget can also show groups of bars via [`BarGroup`].
/// A [`BarGroup`] is a set of [`Bar`], multiple can be added to a `BarChart` using
/// [`BarChart::data`] multiple time as demonstrated in the example below.
///
/// The chart can have a [`Direction`] (by default the bars are [`Vertical`](Direction::Vertical)).
/// This is set using [`BarChart::direction`].
///
/// Note: this is the only widget that doesn't implement `Widget` for `&T` because the current
/// implementation modifies the internal state of self. This will be fixed in the future.
///
/// # Examples
///
/// The following example creates a `BarChart` with two groups of bars.
/// The first group is added by an array slice (`&[(&str, u64)]`).
/// The second group is added by a [`BarGroup`] instance.
/// ```
/// use ratatui::{prelude::*, widgets::*};
///
/// BarChart::default()
///     .block(Block::bordered().title("BarChart"))
///     .bar_width(3)
///     .bar_gap(1)
///     .group_gap(3)
///     .bar_style(Style::new().yellow().on_red())
///     .value_style(Style::new().red().bold())
///     .label_style(Style::new().white())
///     .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
///     .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]))
///     .max(4);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BarChart<'a> {
    /// Block to wrap the widget in
    block: Option<Block<'a>>,
    /// The width of each bar
    bar_width: u16,
    /// The gap between each bar
    bar_gap: u16,
    /// The gap between each group
    group_gap: u16,
    /// Set of symbols used to display the data
    bar_set: symbols::bar::Set,
    /// Style of the bars
    bar_style: Style,
    /// Style of the values printed at the bottom of each bar
    value_style: Style,
    /// Style of the labels printed under each bar
    label_style: Style,
    /// Style for the widget
    style: Style,
    /// vector of groups containing bars
    data: Vec<BarGroup<'a>>,
    /// Value necessary for a bar to reach the maximum height (if no value is specified,
    /// the maximum value in the data is taken as reference)
    max: Option<u64>,
    /// direction of the bars
    direction: Direction,
}

impl<'a> Default for BarChart<'a> {
    fn default() -> Self {
        Self {
            block: None,
            max: None,
            data: Vec::new(),
            bar_style: Style::default(),
            bar_width: 1,
            bar_gap: 1,
            value_style: Style::default(),
            label_style: Style::default(),
            group_gap: 0,
            bar_set: symbols::bar::NINE_LEVELS,
            style: Style::default(),
            direction: Direction::Vertical,
        }
    }
}

impl<'a> BarChart<'a> {
    /// Add group of bars to the `BarChart`
    ///
    /// # Examples
    ///
    /// The following example creates a `BarChart` with two groups of bars.
    /// The first group is added by an array slice (`&[(&str, u64)]`).
    /// The second group is added by a [`BarGroup`] instance.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// BarChart::default()
    ///     .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
    ///     .data(BarGroup::default().bars(&[Bar::default().value(10), Bar::default().value(20)]));
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn data(mut self, data: impl Into<BarGroup<'a>>) -> Self {
        let group: BarGroup = data.into();
        if !group.bars.is_empty() {
            self.data.push(group);
        }
        self
    }

    /// Surround the [`BarChart`] with a [`Block`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the value necessary for a [`Bar`] to reach the maximum height.
    ///
    /// If not set, the maximum value in the data is taken as reference.
    ///
    /// # Examples
    ///
    /// This example shows the default behavior when `max` is not set.
    /// The maximum value in the dataset is taken (here, `100`).
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// BarChart::default().data(&[("foo", 1), ("bar", 2), ("baz", 100)]);
    /// // Renders
    /// //     █
    /// //     █
    /// // f b b
    /// ```
    ///
    /// This example shows a custom max value.
    /// The maximum height being `2`, `bar` & `baz` render as the max.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// BarChart::default()
    ///     .data(&[("foo", 1), ("bar", 2), ("baz", 100)])
    ///     .max(2);
    /// // Renders
    /// //   █ █
    /// // █ █ █
    /// // f b b
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set the default style of the bar.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// It is also possible to set individually the style of each [`Bar`].
    /// In this case the default style will be patched by the individual style
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn bar_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.bar_style = style.into();
        self
    }

    /// Set the width of the displayed bars.
    ///
    /// For [`Horizontal`](crate::layout::Direction::Horizontal) bars this becomes the height of
    /// the bar.
    ///
    /// If not set, this defaults to `1`.
    /// The bar label also uses this value as its width.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width;
        self
    }

    /// Set the gap between each bar.
    ///
    /// If not set, this defaults to `1`.
    /// The bar label will never be larger than the bar itself, even if the gap is sufficient.
    ///
    /// # Example
    ///
    /// This shows two bars with a gap of `3`. Notice the labels will always stay under the bar.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// BarChart::default()
    ///     .data(&[("foo", 1), ("bar", 2)])
    ///     .bar_gap(3);
    /// // Renders
    /// //     █
    /// // █   █
    /// // f   b
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn bar_gap(mut self, gap: u16) -> Self {
        self.bar_gap = gap;
        self
    }

    /// The [`bar::Set`](crate::symbols::bar::Set) to use for displaying the bars.
    ///
    /// If not set, the default is [`bar::NINE_LEVELS`](crate::symbols::bar::NINE_LEVELS).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn bar_set(mut self, bar_set: symbols::bar::Set) -> Self {
        self.bar_set = bar_set;
        self
    }

    /// Set the default value style of the bar.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// It is also possible to set individually the value style of each [`Bar`].
    /// In this case the default value style will be patched by the individual value style
    ///
    /// # See also
    ///
    /// [`Bar::value_style`] to set the value style individually.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn value_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.value_style = style.into();
        self
    }

    /// Set the default label style of the groups and bars.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// It is also possible to set individually the label style of each [`Bar`] or [`BarGroup`].
    /// In this case the default label style will be patched by the individual label style
    ///
    /// # See also
    ///
    /// [`Bar::label`] to set the label style individually.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn label_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.label_style = style.into();
        self
    }

    /// Set the gap between [`BarGroup`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn group_gap(mut self, gap: u16) -> Self {
        self.group_gap = gap;
        self
    }

    /// Set the style of the entire chart.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// The style will be applied to everything that isn't styled (borders, bars, labels, ...).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Set the direction of the bars.
    ///
    /// [`Vertical`](crate::layout::Direction::Vertical) bars are the default.
    ///
    /// # Examples
    ///
    /// Vertical bars
    /// ```plain
    ///   █
    /// █ █
    /// f b
    /// ```
    ///
    /// Horizontal bars
    /// ```plain
    /// █foo██
    ///
    /// █bar██
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

#[derive(Clone, Copy)]
struct LabelInfo {
    group_label_visible: bool,
    bar_label_visible: bool,
    height: u16,
}

impl BarChart<'_> {
    /// Returns the visible bars length in ticks. A cell contains 8 ticks.
    /// `available_space` used to calculate how many bars can fit in the space
    /// `bar_max_length` is the maximal length a bar can take.
    fn group_ticks(&self, available_space: u16, bar_max_length: u16) -> Vec<Vec<u64>> {
        let max: u64 = self.maximum_data_value();
        self.data
            .iter()
            .scan(available_space, |space, group| {
                if *space == 0 {
                    return None;
                }
                let n_bars = group.bars.len() as u16;
                let group_width = n_bars * self.bar_width + n_bars.saturating_sub(1) * self.bar_gap;

                let n_bars = if *space > group_width {
                    *space = space.saturating_sub(group_width + self.group_gap + self.bar_gap);
                    Some(n_bars)
                } else {
                    let max_bars = (*space + self.bar_gap) / (self.bar_width + self.bar_gap);
                    if max_bars > 0 {
                        *space = 0;
                        Some(max_bars)
                    } else {
                        None
                    }
                };

                n_bars.map(|n| {
                    group
                        .bars
                        .iter()
                        .take(n as usize)
                        .map(|bar| bar.value * u64::from(bar_max_length) * 8 / max)
                        .collect()
                })
            })
            .collect()
    }

    /// Get label information.
    ///
    /// height is the number of lines, which depends on whether we need to print the bar
    /// labels and/or the group labels.
    /// - If there are no labels, height is 0.
    /// - If there are only bar labels, height is 1.
    /// - If there are only group labels, height is 1.
    /// - If there are both bar and group labels, height is 2.
    fn label_info(&self, available_height: u16) -> LabelInfo {
        if available_height == 0 {
            return LabelInfo {
                group_label_visible: false,
                bar_label_visible: false,
                height: 0,
            };
        }

        let bar_label_visible = self
            .data
            .iter()
            .any(|e| e.bars.iter().any(|e| e.label.is_some()));

        if available_height == 1 && bar_label_visible {
            return LabelInfo {
                group_label_visible: false,
                bar_label_visible: true,
                height: 1,
            };
        }

        let group_label_visible = self.data.iter().any(|e| e.label.is_some());
        LabelInfo {
            group_label_visible,
            bar_label_visible,
            // convert true to 1 and false to 0 and add the two values
            height: u16::from(group_label_visible) + u16::from(bar_label_visible),
        }
    }

    fn render_horizontal(&self, buf: &mut Buffer, area: Rect) {
        // get the longest label
        let label_size = self
            .data
            .iter()
            .flat_map(|group| group.bars.iter().map(|bar| &bar.label))
            .flatten() // bar.label is an Option<Line>
            .map(Line::width)
            .max()
            .unwrap_or(0) as u16;

        let label_x = area.x;
        let bars_area = {
            let margin = u16::from(label_size != 0);
            Rect {
                x: area.x + label_size + margin,
                width: area.width - label_size - margin,
                ..area
            }
        };

        let group_ticks = self.group_ticks(bars_area.height, bars_area.width);

        // print all visible bars, label and values
        let mut bar_y = bars_area.top();
        for (ticks_vec, group) in group_ticks.into_iter().zip(self.data.iter()) {
            for (ticks, bar) in ticks_vec.into_iter().zip(group.bars.iter()) {
                let bar_length = (ticks / 8) as u16;
                let bar_style = self.bar_style.patch(bar.style);

                for y in 0..self.bar_width {
                    let bar_y = bar_y + y;
                    for x in 0..bars_area.width {
                        let symbol = if x < bar_length {
                            self.bar_set.full
                        } else {
                            self.bar_set.empty
                        };
                        buf[(bars_area.left() + x, bar_y)]
                            .set_symbol(symbol)
                            .set_style(bar_style);
                    }
                }

                let bar_value_area = Rect {
                    y: bar_y + (self.bar_width >> 1),
                    ..bars_area
                };

                // label
                if let Some(label) = &bar.label {
                    buf.set_line(label_x, bar_value_area.top(), label, label_size);
                }

                bar.render_value_with_different_styles(
                    buf,
                    bar_value_area,
                    bar_length as usize,
                    self.value_style,
                    self.bar_style,
                );

                bar_y += self.bar_gap + self.bar_width;
            }

            // if group_gap is zero, then there is no place to print the group label
            // check also if the group label is still inside the visible area
            let label_y = bar_y - self.bar_gap;
            if self.group_gap > 0 && label_y < bars_area.bottom() {
                let label_rect = Rect {
                    y: label_y,
                    ..bars_area
                };
                group.render_label(buf, label_rect, self.label_style);
                bar_y += self.group_gap;
            }
        }
    }

    fn render_vertical(&self, buf: &mut Buffer, area: Rect) {
        let label_info = self.label_info(area.height - 1);

        let bars_area = Rect {
            height: area.height - label_info.height,
            ..area
        };

        let group_ticks = self.group_ticks(bars_area.width, bars_area.height);
        self.render_vertical_bars(bars_area, buf, &group_ticks);
        self.render_labels_and_values(area, buf, label_info, &group_ticks);
    }

    fn render_vertical_bars(&self, area: Rect, buf: &mut Buffer, group_ticks: &[Vec<u64>]) {
        // print all visible bars (without labels and values)
        let mut bar_x = area.left();
        for (ticks_vec, group) in group_ticks.iter().zip(&self.data) {
            for (ticks, bar) in ticks_vec.iter().zip(&group.bars) {
                let mut ticks = *ticks;
                for j in (0..area.height).rev() {
                    let symbol = match ticks {
                        0 => self.bar_set.empty,
                        1 => self.bar_set.one_eighth,
                        2 => self.bar_set.one_quarter,
                        3 => self.bar_set.three_eighths,
                        4 => self.bar_set.half,
                        5 => self.bar_set.five_eighths,
                        6 => self.bar_set.three_quarters,
                        7 => self.bar_set.seven_eighths,
                        _ => self.bar_set.full,
                    };

                    let bar_style = self.bar_style.patch(bar.style);

                    for x in 0..self.bar_width {
                        buf[(bar_x + x, area.top() + j)]
                            .set_symbol(symbol)
                            .set_style(bar_style);
                    }

                    ticks = ticks.saturating_sub(8);
                }
                bar_x += self.bar_gap + self.bar_width;
            }
            bar_x += self.group_gap;
        }
    }

    /// get the maximum data value. the returned value is always greater equal 1
    fn maximum_data_value(&self) -> u64 {
        self.max
            .unwrap_or_else(|| {
                self.data
                    .iter()
                    .map(|group| group.max().unwrap_or_default())
                    .max()
                    .unwrap_or_default()
            })
            .max(1)
    }

    fn render_labels_and_values(
        &self,
        area: Rect,
        buf: &mut Buffer,
        label_info: LabelInfo,
        group_ticks: &[Vec<u64>],
    ) {
        // print labels and values in one go
        let mut bar_x = area.left();
        let bar_y = area.bottom() - label_info.height - 1;
        for (group, ticks_vec) in self.data.iter().zip(group_ticks) {
            if group.bars.is_empty() {
                continue;
            }
            // print group labels under the bars or the previous labels
            if label_info.group_label_visible {
                let label_max_width =
                    ticks_vec.len() as u16 * (self.bar_width + self.bar_gap) - self.bar_gap;
                let group_area = Rect {
                    x: bar_x,
                    y: area.bottom() - 1,
                    width: label_max_width,
                    height: 1,
                };
                group.render_label(buf, group_area, self.label_style);
            }

            // print the bar values and numbers
            for (bar, ticks) in group.bars.iter().zip(ticks_vec) {
                if label_info.bar_label_visible {
                    bar.render_label(buf, self.bar_width, bar_x, bar_y + 1, self.label_style);
                }

                bar.render_value(buf, self.bar_width, bar_x, bar_y, self.value_style, *ticks);

                bar_x += self.bar_gap + self.bar_width;
            }
            bar_x += self.group_gap;
        }
    }
}

impl Widget for BarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for BarChart<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        self.block.render_ref(area, buf);
        let inner = self.block.inner_if_some(area);

        if inner.is_empty() || self.data.is_empty() || self.bar_width == 0 {
            return;
        }

        match self.direction {
            Direction::Horizontal => self.render_horizontal(buf, inner),
            Direction::Vertical => self.render_vertical(buf, inner),
        }
    }
}

impl<'a> Styled for BarChart<'a> {
    type Item = Self;
    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}
