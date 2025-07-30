// This file is part of `iced_experimental` crate. For the terms of use, please see the file
// called LICENSE-BSD-3-Clause at the top level of the `iced_experimental` crate.

//! The style file for cell widget.

#[doc(inline)]
#[allow(unused_imports)]
use super::cell::Cell;

use crate::core::{Background, Border, Color, Theme, border::Radius};
use iced_widget::text_input;

/// The appearance of a cell container.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The text [`Color`] of the container.
    pub text_color: Color,
    /// The [`Background`] of the container.
    pub background: Background,
    /// The [`Border`] of the container.
    pub border: Border,
}

impl Style {
    /// Updates the [`Style`] with the given [`Color`].
    pub fn text_color(self, text_color: impl Into<Color>) -> Self {
        Self {
            text_color: text_color.into(),
            ..self
        }
    }

    /// Updates the [`Style`] with the given [`Background`].
    pub fn background(self, background: impl Into<Background>) -> Self {
        Self {
            background: background.into(),
            ..self
        }
    }

    /// Updates the [`Style`] with the given [`Border`].
    pub fn border(self, border: impl Into<Border>) -> Self {
        Self {
            border: border.into(),
            ..self
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            text_color: Color::default(),
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
        }
    }
}

/// Style selecting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Styling {
    /// The [`Cell`] is a label.
    Label,
    /// The divider style of the labels.
    Divider(bool), // True indicates is hovered.
    /// The [`Cell`] is read only value.
    ReadOnly,
    /// The [`Cell`] is a value.
    Value(bool), // True indicates value is not saved.
    /// The [`Cell`] is a value, though with alternating background.
    RowAlternating(bool, usize), // The `uszie` is row number.
}

/// The theme catalog of cell grid widgets.
pub trait Catalog {
    /// Select a specific style to apply to the widget.
    fn style(&self, styling: Styling) -> Style {
        match styling {
            Styling::Label => Self::label(self),
            Styling::ReadOnly => Self::read_only(self),
            Styling::Divider(hovered) => Self::divider(self, hovered),
            Styling::Value(changed) => Self::value(self, changed),
            Styling::RowAlternating(changed, index) => Self::row_alternating(self, changed, index),
        }
    }

    /// The style for the labels.
    fn label(&self) -> Style;

    /// The style for read only cells.
    fn read_only(&self) -> Style;

    /// The style for editable value.
    fn value(&self, changed: bool) -> Style;

    /// The style for editable value with alternative row background color.
    fn row_alternating(&self, changed: bool, index: usize) -> Style;

    /// The style for the label divider.
    fn divider(&self, hovered: bool) -> Style;
}

impl Catalog for Theme {
    fn label(&self) -> Style {
        let extended = self.extended_palette();
        Style {
            background: Background::Color(extended.secondary.base.color),
            border: Border {
                color: self.palette().text,
                width: 1.0,
                radius: Radius::new(0),
            },
            ..Style::default()
        }
    }

    fn read_only(&self) -> Style {
        let extended = self.extended_palette();
        Style {
            background: Background::Color(extended.primary.base.color),
            border: Border {
                color: self.palette().text,
                width: 1.0,
                radius: Radius::new(0),
            },
            ..Style::default()
        }
    }

    fn value(&self, changed: bool) -> Style {
        let palette = self.palette();
        let extended = self.extended_palette();
        let border = if changed {
            palette.danger
            //extended.danger.base.color
        } else {
            palette.text
        };
        Style {
            background: Background::Color(extended.background.base.color),
            border: Border {
                color: border,
                width: 1.0,
                radius: Radius::new(0),
            },
            ..Style::default()
        }
    }

    fn row_alternating(&self, changed: bool, index: usize) -> Style {
        let palette = self.palette();
        let extended = self.extended_palette();
        let border = if changed {
            palette.danger
            //extended.danger.base.color
        } else {
            palette.text
        };
        let background = if index % 2 == 0 {
            extended.background.base.color
        } else {
            extended.background.weak.color
        };
        Style {
            background: background.into(),
            border: Border {
                color: border,
                width: 1.0,
                radius: Radius::new(0),
            },
            ..Style::default()
        }
    }

    fn divider(&self, hovered: bool) -> Style {
        let extended = self.extended_palette();
        let background = if hovered {
            extended.primary.base.color
        } else {
            extended.background.weak.color
        };
        Style {
            background: background.into(),
            ..Style::default()
        }
    }
}

//
//
// ----- Extends the [`TextInput`] catalog [`text_input::Catalog`]
//
//

pub trait TextInputCatalog: text_input::Catalog {
    /// Adding a no border style to [`text_input::Catalog`]
    fn no_border<'a>() -> <Self as text_input::Catalog>::Class<'a>;
}

impl TextInputCatalog for Theme {
    fn no_border<'a>() -> <Self as text_input::Catalog>::Class<'a> {
        Box::new(text_input_no_border)
    }
}

pub fn text_input_no_border(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::default(theme, status);
    text_input::Style {
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(0.0),
        },
        ..base
    }
}
