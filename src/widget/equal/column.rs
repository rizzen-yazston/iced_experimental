// This file is part of `iced_experimental` crate. For the terms of use, please see the file
// called LICENSE-BSD-3-Clause at the top level of the `iced_experimental` crate.

//! A container that distributes its contents vertically, while ensuring all
//! the children have equal widths, by expanding the children widths to the
//! child with the widest width.
//!
//! [`EqualWidthColumn`] differs from [`Column`] of `iced_widget` crate in the
//! following way:
//!
//! * Intended to have alignment container widgets as its children, as this
//!   widget alters the widths of the children.
//!
//! * No support for [`Length::Fill`] or [`Length::FillPortion`], as `Column`
//!   already ensures the children fills the allowable width.
//!
//! * [`Length::Fixed`] is support by using [`EqualWidthColumn::fixed_width()`],
//!   and when used on the widget width the fixed value is used for all children
//!   widths.
//!
//! * When using [`Length::Shrink`] (default), there are additional options:
//!
//! ** Set optional maximum width, allowing extra flexibility than hard fixed
//!    width using [`Length::Fixed`].
//!
//! ** Set optional maximum height, allowing extra flexibility than hard fixed
//!    height using [`Length::Fixed`].
//!
//! ** Set optional minimum width, allowing extra flexibility than hard fixed
//!    width using [`Length::Fixed`], but also allows entire column to be
//!    aligned horizontally if content is smaller than the minimum width.
//!
//! ** Set optional minimum height, allowing extra flexibility than hard fixed
//!    height using [`Length::Fixed`], but also allows entire column to be
//!    aligned vertically if content is smaller than the minimum height.
//!
//! * The ability to reverse the current children of the widget.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{container};
//! use iced_experimental_rizzen_yazston::widget::EqualWidthColumn;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     EqualWidthColumn::with_children([
//!         container("Top element").into(),
//!         container("Centre element").into(),
//!         container("Bottom element").into(),
//!     ]).into()
//! }
//! ```

#[doc(inline)]
#[allow(unused_imports)]
use iced_widget::Column;

use crate::core::{
    Clipboard,
    Element,
    Event,
    Length,
    Padding,
    Pixels,
    Point,
    Rectangle,
    Shell,
    Size,
    Vector,
    alignment,
    layout::{self, Layout, Node},
    mouse,
    overlay,
    renderer,
    widget::{Operation, Tree, Widget}, // operate = iced::runtime::widget
};

/// Widget for column of equal width entries. Ensures the children are resized.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{container};
/// use iced_experimental_rizzen_yazston::widget::EqualWidthColumn;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     EqualWidthColumn::with_children([
///         container("Top element").into(),
///         container("Centre element").into(),
///         container("Bottom element").into(),
///     ]).into()
/// }
/// ```
pub struct EqualWidthColumn<'a, Message, Theme, Renderer> {
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    max_width: Option<f32>,
    max_height: Option<f32>,
    min_width: Option<f32>,
    min_height: Option<f32>,
    vertical: alignment::Vertical,
    horizontal: alignment::Horizontal,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Default for EqualWidthColumn<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> EqualWidthColumn<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
{
    /// Creates a [`EqualWidthColumn`] from an already allocated [`Vec`].
    pub fn from_vec(children: Vec<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: None,
            max_height: None,
            min_width: None,
            min_height: None,
            vertical: alignment::Vertical::Top,
            horizontal: alignment::Horizontal::Left,
            children,
        }
    }

    /// Creates an empty [`EqualWidthColumn`].
    pub fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Creates a [`EqualWidthColumn`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_vec(Vec::with_capacity(capacity))
    }

    /// Creates a [`EqualWidthColumn`] with the given elements.
    pub fn with_children(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let iterator = children.into_iter();

        Self::with_capacity(iterator.size_hint().0).extend(iterator)
    }

    /// Extends the [`EqualWidthColumn`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }

    /// Adds an [`Element`] to the [`EqualWidthColumn`].
    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();
        self.width = self.width.enclose(child_size.width);
        self.height = self.height.enclose(child_size.height);
        self.children.push(child);
        self
    }

    /// Adds an element to the [`EqualWidthColumn`], if `Some`.
    pub fn push_maybe(
        self,
        child: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Self {
        if let Some(child) = child {
            self.push(child)
        } else {
            self
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`EqualWidthColumn`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the height of the [`EqualWidthColumn`].
    pub fn fixed_height(mut self, height: Option<f32>) -> Self {
        self.height = match height {
            None => Length::Shrink,
            Some(value) => Length::Fixed(value),
        };
        self
    }

    /// Sets the width of the [`EqualWidthColumn`].
    pub fn fixed_width(mut self, width: Option<f32>) -> Self {
        self.width = match width {
            None => Length::Shrink,
            Some(value) => Length::Fixed(value),
        };
        self
    }

    /// Sets the maximum height of the [`EqualWidthColumn`].
    pub fn max_height(mut self, height: Option<f32>) -> Self {
        self.max_height = height;
        self
    }

    /// Sets the maximum width of the [`EqualWidthColumn`].
    pub fn max_width(mut self, width: Option<f32>) -> Self {
        self.max_width = width;
        self
    }

    /// Sets the minimum width of the [`EqualWidthColumn`].
    pub fn min_width(mut self, width: Option<f32>) -> Self {
        self.min_width = width;
        self
    }

    /// Sets the minimum height of the [`EqualWidthColumn`].
    pub fn min_height(mut self, height: Option<f32>) -> Self {
        self.min_height = height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`EqualWidthColumn`].
    pub fn align_x(mut self, align: impl Into<alignment::Alignment>) -> Self {
        self.horizontal = alignment::Horizontal::from(align.into());
        self
    }

    /// Sets the vertical alignment of the contents of the [`EqualWidthColumn`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Alignment>) -> Self {
        self.vertical = alignment::Vertical::from(alignment.into());
        self
    }

    /// Reverse the order of the existing children [`EqualWidthColumn`].
    pub fn reverse(mut self) -> Self {
        self.children.reverse();
        self
    }
}

impl<'a, Message, Theme, Renderer> FromIterator<Element<'a, Message, Theme, Renderer>>
    for EqualWidthColumn<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: 'a,
{
    fn from_iter<T: IntoIterator<Item = Element<'a, Message, Theme, Renderer>>>(iter: T) -> Self {
        Self::with_children(iter)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for EqualWidthColumn<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> Node {
        let node = layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            self.horizontal.into(),
            &self.children,
            &mut tree.children,
        );
        let mut width = node.size().width;
        let mut width_diff = 0.0f32;
        if self.width == Length::Shrink {
            if self.max_width.is_some() {
                width = width.min(self.max_width.unwrap());
            }
            if self.min_width.is_some() {
                let min_width = self.min_width.unwrap();
                width = width.max(self.min_width.unwrap());
                match self.horizontal {
                    alignment::Horizontal::Center => width_diff = (min_width - width) / 2.0,
                    alignment::Horizontal::Right => width_diff = min_width - width,
                    _ => {}
                }
            }
        }
        let mut height = node.size().height;
        let mut height_diff = 0.0f32;
        if self.height == Length::Shrink {
            if self.max_height.is_some() {
                height = height.min(self.max_height.unwrap());
            }
            if self.min_height.is_some() {
                let min_height = self.min_height.unwrap();
                height = height.max(self.min_height.unwrap());
                match self.vertical {
                    alignment::Vertical::Center => height_diff = (min_height - height) / 2.0,
                    alignment::Vertical::Bottom => height_diff = min_height - height,
                    _ => {}
                }
            }
        }
        let mut children = Vec::<Node>::new();
        for child in node.children().iter() {
            let content_width = node.size().width - self.padding.left - self.padding.right;
            let child_diff = content_width - child.size().width;
            let child_x = child.bounds().x
                + match self.horizontal {
                    alignment::Horizontal::Left => 0.0,
                    alignment::Horizontal::Center => child_diff / 2.0,
                    alignment::Horizontal::Right => child_diff,
                };
            let mut new_child = if child_diff == 0.0 {
                Node::with_children(child.size(), child.children().to_owned())
            } else {
                Node::with_children(
                    Size::new(content_width, child.size().height),
                    child.children().to_owned(),
                )
            };
            new_child.move_to_mut(Point::new(
                child_x + width_diff,
                child.bounds().y + height_diff,
            ));
            children.push(new_child);
        }
        Node::with_children(Size::new(width, height), children)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let _ = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().update(
                    state, event, layout, cursor, renderer, clipboard, shell, viewport,
                )
            });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<EqualWidthColumn<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(row: EqualWidthColumn<'a, Message, Theme, Renderer>) -> Self {
        Self::new(row)
    }
}
