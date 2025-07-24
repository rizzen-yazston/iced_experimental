// This file is part of `iced_experimental` project. For the terms of use, please see the file
// called `LICENSE-BSD-3-Clause` at the top level of the `iced_experimental` project's Git repository.

pub mod scrollable;
pub use scrollable::{Scrollable, Scrollbar};

pub mod equal;
pub use equal::{column::EqualWidthColumn, row::EqualHeightRow};

use crate::core::{self, Element};

/// Creates a new [`Scrollable`] with the provided content.
///
/// Scrollables let users navigate an endless amount of content with a scrollbar.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, scrollable, vertical_space};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     scrollable(column![
///         "Scroll me!",
///         vertical_space().height(3000),
///         "You did it!",
///     ]).into()
/// }
/// ```
pub fn scrollable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: scrollable::Catalog + 'a,
    Renderer: core::Renderer,
{
    Scrollable::new(content)
}
