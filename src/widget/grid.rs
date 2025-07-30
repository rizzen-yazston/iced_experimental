// This file is part of `iced_experimental` crate. For the terms of use, please see the file
// called LICENSE-BSD-3-Clause at the top level of the `iced_experimental` crate.

//! A container that distributes its contents in a grid of variable column
//! widths and variable row heights.
//!
//! [`Grid`] differs from `Grid` of `iced_widget` crate in the following way:
//!
//! * Uses a different design approach.
//!
//! * Intended to have alignment container widgets as its children, as this
//!   widget alters the widths and heights of the children.
//!
//! * Supports padding around the entire widget.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{container};
//! use iced_experimental_rizzen_yazston::widget::Grid;
//!
//! #[derive(Clone)]
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     Grid::from_vec(
//!         vec![
//!             container("Top left element").into(),
//!             container("Top right element").into(),
//!             container("Bottom left element").into(),
//!             container("Bottom right element").into(),
//!         ],
//!         vec![100.0, 100.0],
//!         vec![16.0, 16.0],
//!     ).into()
//! }
//! ```

use crate::core::{
    Clipboard,
    Element,
    Event,
    Length,
    Pixels,
    Point,
    Rectangle,
    Shell,
    Size,
    Vector,
    layout::{self, Layout, Limits, Node},
    mouse,
    overlay,
    renderer,
    widget::{Operation, Tree, Widget}, // operate = iced::runtime::widget
};
use iced_widget::{Space, core::Padding};

/// A container that distributes its contents in a grid of variable column
/// widths and variable row heights.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::container;
/// use iced_experimental_rizzen_yazston::widget::Grid;
///
/// #[derive(Clone)]
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     Grid::from_vec(
///         vec![
///             container("Top left element").into(),
///             container("Top right element").into(),
///             container("Bottom left element").into(),
///             container("Bottom right element").into(),
///         ],
///         vec![100.0, 100.0],
///         vec![16.0, 16.0],
///     ).into()
/// }
/// ```
pub struct Grid<'a, Message, Theme, Renderer> {
    spacing: f32,
    padding: Padding,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    column_widths: Vec<f32>,
    row_heights: Vec<f32>,
}

impl<'a, Message, Theme, Renderer> Default for Grid<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Message: 'a,
{
    /// Creates a [`Grid`] from an already allocated [`Vec`].
    ///
    /// Note: The children vector will be truncated to the multiplied lengths of
    /// the other two vectors.
    pub fn from_vec(
        children: Vec<Element<'a, Message, Theme, Renderer>>,
        column_widths: Vec<f32>,
        row_heights: Vec<f32>,
    ) -> Self {
        let mut actual = children;
        actual.truncate(column_widths.len() * row_heights.len());
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            children: actual,
            column_widths,
            row_heights,
        }
    }

    /// Creates an empty [`Grid`].
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            children: Vec::new(),
            column_widths: Vec::new(),
            row_heights: Vec::new(),
        }
    }

    /// Creates an empty [`Grid`] with the given capacity.
    pub fn with_capacity(columns: usize, rows: usize) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            children: Vec::with_capacity(rows * columns),
            column_widths: Vec::with_capacity(columns),
            row_heights: Vec::with_capacity(rows),
        }
    }

    /// Adds an [`Element`] to the [`Grid`].
    ///
    /// Note: Pushing an [`Element`] beyond the length of the cells capacity
    /// (length of the column widths multiplied by length of the row heights) will
    /// result in the element simply be discarded. Ensure the pushing of all column
    /// widths and all row heights have been completed before starting to push the
    /// cell elements in order to avoid rendering errors of the cells.
    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        if self.children.len() < self.column_widths.len() * self.row_heights.len() {
            self.children.push(child.into());
        }
        self
    }

    /// Adds a column width to the [`Grid`].
    pub fn push_column_width(mut self, child: impl Into<f32>) -> Self {
        self.column_widths.push(child.into());
        self
    }

    /// Adds a row height to the [`Grid`].
    pub fn push_row_height(mut self, child: impl Into<f32>) -> Self {
        self.row_heights.push(child.into());
        self
    }

    /// Fill remaining cells of the [`Grid`] with [`Space`] widgets, if any.
    pub fn fill(mut self) -> Self {
        let total = self.column_widths.len() * self.row_heights.len();
        while self.children.len() < total {
            // Arbitrary width and height used as they will be resized to fit the cell's
            // dimensions.
            self.children.push(Space::new(1.0, 1.0).into());
        }
        self
    }

    /// Sets the horizontal and vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`Grid`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Grid<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, _limits: &layout::Limits) -> Node {
        // The layout of the children is done in top to down rows of left to right
        // ordering. i.e. Latin scripts.

        // Obtain the column positions of the cells within the rows.
        let mut columns = Vec::<(f32, f32)>::new();
        let mut grid_width = self.padding.left;
        for (index, value) in self.column_widths.iter().enumerate() {
            if index > 0 {
                grid_width += self.spacing;
            }
            columns.push((grid_width, *value));
            grid_width += value;
        }

        // Build the node tree
        let mut nodes =
            Vec::<Node>::with_capacity(self.row_heights.len() * self.column_widths.len());
        let mut index = 0usize;
        let mut grid_height = self.padding.top;
        for (row, height) in self.row_heights.iter().enumerate() {
            if row > 0 {
                grid_height += self.spacing;
            }
            for (x, width) in &columns {
                let size = Size {
                    width: *width,
                    height: *height,
                };
                let node = self.children[index].as_widget().layout(
                    &mut tree.children[index],
                    renderer,
                    &Limits::new(Size::ZERO, size),
                );
                let children = node.children();
                let mut child: Node = if children.is_empty() {
                    Node::new(size)
                } else {
                    Node::with_children(size, children.to_vec())
                };
                child.move_to_mut(Point {
                    x: *x,
                    y: grid_height,
                });
                nodes.push(child);
                index += 1;
            }
            grid_height += height;
        }
        Node::with_children(
            Size::new(
                grid_width + self.padding.right,
                grid_height + self.padding.bottom,
            ),
            nodes,
        )
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
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            )
        }
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

impl<'a, Message, Theme, Renderer> From<Grid<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: 'a,
{
    fn from(grid: Grid<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        // Ensure every cell has a widget.
        let mut grid = grid;
        grid = grid.fill();
        Element::new(grid)
    }
}
