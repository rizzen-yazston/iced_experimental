// This file is part of `iced_experimental` crate. For the terms of use, please see the file
// called LICENSE-BSD-3-Clause at the top level of the `iced_experimental` crate.

//! [`Cell`] is a container, that may be clickable (for dynamic content cells),
//! be resizable with displaying resize mouse pointer, specify the alignment
//! direction of the content within the cell, and change the border colour when
//! the content has changed. Also supports having alternative background when
//! used in a grid of cells, such as alternative row background colouring.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::text;
//! use iced_experimental_rizzen_yazston::widget::{cell, cell::style};
//!
//! #[derive(Clone)]
//! enum Message {
//!     Pressed(usize, usize),
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     cell(
//!         true,
//!         text("Text content of the cell.").into(),
//!     )
//!     .on_press(Message::Pressed(5, 8))
//!     .styling(&style::Styling::Value(true))
//!     .into()
//! }
//! ```

/*
#[doc(inline)]
#[allow(unused_imports)]
use iced_widget::Text;
*/

use super::style;
use crate::core::{
    Border,
    Clipboard,
    Color,
    Element,
    Event,
    Length,
    Padding,
    Point,
    Rectangle,
    Shadow,
    Shell,
    Size,
    Vector,
    alignment,
    layout::{self, Layout},
    mouse,
    overlay,
    renderer,
    touch,
    widget::{Operation, Tree, Widget, tree}, // operate = iced::runtime::widget
};

/// Helper function for creating [`Cell`] instance.
pub fn cell<'a, Message, Theme, Renderer>(
    clickable: bool,
    content: Element<'a, Message, Theme, Renderer>,
) -> Cell<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: style::Catalog + 'a,
    Message: std::clone::Clone + 'a,
{
    Cell::new(clickable, content)
}

/// [`Cell`] is a container, that may be clickable (for dynamic content cells),
/// be resizable with displaying resize mouse pointer, specify the alignment
/// direction of the content within the cell, and change the border colour when
/// the content has changed. Also supports having alternative background when
/// used in a grid of cells, such as alternative row background colouring.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::text;
/// use iced_experimental_rizzen_yazston::widget::{cell, cell::style};
///
/// #[derive(Clone)]
/// enum Message {
///     Pressed(usize, usize),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     cell(
///         true,
///         text("Text content of the cell.").into(),
///     )
///     .on_press(Message::Pressed(5, 8))
///     .styling(&style::Styling::Value(true))
///     .into()
/// }
/// ```
pub struct Cell<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Theme: style::Catalog,
{
    // Messages
    on_press: Option<Message>,
    #[allow(clippy::type_complexity)]
    on_resize_horizontal: Option<(Box<dyn Fn(f32) -> Message + 'a>, Message)>,
    #[allow(clippy::type_complexity)]
    on_resize_vertical: Option<(Box<dyn Fn(f32) -> Message + 'a>, Message)>,

    // Layout
    padding: Padding,
    height_resize_offset: Option<f32>,
    width_resize_offset: Option<f32>,
    align_x: alignment::Horizontal,
    align_y: alignment::Vertical,
    resize_hover_size: f32,

    // Content
    clickable: bool,
    content: Element<'a, Message, Theme, Renderer>,

    // Styling
    styling: style::Styling,
    change_color: Option<Color>,
}

impl<'a, Message, Theme, Renderer> Cell<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Theme: style::Catalog + 'a,
    Message: std::clone::Clone + 'a,
{
    /// Creates a [`Cell`] with the given content.
    ///
    /// Parameter `clickable`: Sets whether cell is clickable.
    ///
    /// Parameter `content`: Content widget of the cell.
    pub fn new(clickable: bool, content: Element<'a, Message, Theme, Renderer>) -> Self {
        Cell {
            // Messages
            on_press: None,
            on_resize_horizontal: None,
            on_resize_vertical: None,

            // Layout
            padding: Padding::new(2.0),
            height_resize_offset: None,
            width_resize_offset: None,
            align_x: alignment::Horizontal::Left,
            align_y: alignment::Vertical::Top,
            resize_hover_size: 5.0,

            // Content
            clickable,
            content,

            // Styling
            styling: style::Styling::ReadOnly,
            change_color: None,
        }
    }

    /// Sets the message that will be produced when the [`Cell`] is pressed.
    ///
    /// Unless `on_press` is called, the [`Cell`] will be disabled.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets the message that will be produced when a [`Cell`] is resizing. Setting this
    /// will enable the resizing interaction.
    ///
    /// `on_drag` will emit a message during an on-going resize. It is up to the consumer to return
    /// this value for the associated column in [`Cell::width_resize_offset`].
    ///
    /// `on_release` is emited when the resize is finished. It is up to the consumer to apply the last
    /// `on_drag` offset to the column's stored width.
    pub fn on_resize_horizontal(
        mut self,
        on_drag: impl Fn(f32) -> Message + 'a,
        on_release: Message,
    ) -> Self {
        self.on_resize_horizontal = Some((Box::new(on_drag), on_release));
        self
    }

    /// Sets the message that will be produced when a [`Cell`] is resizing. Setting this
    /// will enable the resizing interaction.
    ///
    /// `on_drag` will emit a message during an on-going resize. It is up to the consumer to return
    /// this value for the associated column in [`Cell::height_resize_offset`].
    ///
    /// `on_release` is emited when the resize is finished. It is up to the consumer to apply the last
    /// `on_drag` offset to the column's stored width.
    pub fn on_resize_vertical(
        mut self,
        on_drag: impl Fn(f32) -> Message + 'a,
        on_release: Message,
    ) -> Self {
        self.on_resize_vertical = Some((Box::new(on_drag), on_release));
        self
    }

    /// Sets the [`Padding`] within the [`Cell`]. Must also include border width.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the minimum width of the [`Cell`].
    pub fn height_resize_offset(mut self, offset: &Option<f32>) -> Self {
        self.height_resize_offset = *offset;
        self
    }

    /// Sets the minimum width of the [`Cell`].
    pub fn width_resize_offset(mut self, offset: &Option<f32>) -> Self {
        self.width_resize_offset = *offset;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Cell`].
    pub fn align_x(mut self, alignment: impl Into<alignment::Alignment>) -> Self {
        self.align_x = alignment::Horizontal::from(alignment.into());
        self
    }

    /// Sets the vertical alignment of the contents of the [`Cell`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Alignment>) -> Self {
        self.align_y = alignment::Vertical::from(alignment.into());
        self
    }

    /// Sets the resize detection space on either side of the widget's edge.
    pub fn resize_hover_size(mut self, size: f32) -> Self {
        self.resize_hover_size = size.min(1.0);
        self
    }

    /// Sets the styling of the [`Cell`].
    pub fn styling(mut self, styling: &style::Styling) -> Self {
        self.styling = *styling;
        self
    }

    /// Sets the change border color of the [`Cell`].
    pub fn change_color(mut self, color: Option<Color>) -> Self {
        self.change_color = color;
        self
    }
}

/// Just a smaller persistent state for the `Tree`.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    is_pressed: bool,
    drag_origin_horizontal: Option<Point>,
    is_resize_hovered_horizontal: bool,
    drag_origin_vertical: Option<Point>,
    is_resize_hovered_vertical: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Cell<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Theme: style::Catalog,
    Message: std::clone::Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, Length::Fill, Length::Fill, self.padding, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let resize_hover_bounds_horizontal = Rectangle {
            x: bounds.x + bounds.width - self.resize_hover_size,
            width: 2.0 * self.resize_hover_size,
            ..bounds
        };
        if self.on_resize_horizontal.is_some() {
            state.is_resize_hovered_horizontal = cursor.is_over(resize_hover_bounds_horizontal);
        }
        let resize_hover_bounds_vertical = Rectangle {
            y: bounds.y + bounds.height - self.resize_hover_size,
            height: 2.0 * self.resize_hover_size,
            ..bounds
        };
        if self.on_resize_vertical.is_some() {
            state.is_resize_hovered_vertical = cursor.is_over(resize_hover_bounds_vertical);
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_resize_horizontal.is_some()
                    && let Some(origin) = cursor.position_over(resize_hover_bounds_horizontal)
                {
                    state.drag_origin_horizontal = Some(origin);
                    shell.capture_event();
                }
                if self.on_resize_vertical.is_some()
                    && let Some(origin) = cursor.position_over(resize_hover_bounds_vertical)
                {
                    state.drag_origin_vertical = Some(origin);
                    shell.capture_event();
                }
                let bounds = layout.bounds();
                if cursor.is_over(bounds) {
                    let state = tree.state.downcast_mut::<State>();
                    state.is_pressed = true;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if self.on_resize_horizontal.is_some()
                    && state.drag_origin_horizontal.take().is_some()
                {
                    shell.publish(self.on_resize_horizontal.as_ref().unwrap().1.clone());
                    shell.capture_event();
                }
                if self.on_resize_vertical.is_some() && state.drag_origin_vertical.take().is_some()
                {
                    shell.publish(self.on_resize_vertical.as_ref().unwrap().1.clone());
                    shell.capture_event();
                }
                if state.is_pressed {
                    state.is_pressed = false;
                    let bounds = layout.bounds();
                    if cursor.is_over(bounds) {
                        if let Some(message) = &self.on_press {
                            shell.publish(message.clone());
                        }
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.on_resize_horizontal.is_some() {
                    if let Some(position) = cursor.position() {
                        if let Some(origin) = state.drag_origin_horizontal {
                            shell.publish((self.on_resize_horizontal.as_ref().unwrap().0)(
                                (position - origin).x,
                            ));
                            shell.capture_event();
                        }
                    }
                }
                if self.on_resize_vertical.is_some() {
                    if let Some(position) = cursor.position() {
                        if let Some(origin) = state.drag_origin_vertical {
                            shell.publish((self.on_resize_vertical.as_ref().unwrap().0)(
                                (position - origin).y,
                            ));
                            shell.capture_event();
                        }
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                let state = tree.state.downcast_mut::<State>();
                state.is_pressed = false;
            }
            _ => {}
        };
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.drag_origin_horizontal.is_some() || state.is_resize_hovered_horizontal {
            mouse::Interaction::ResizingHorizontally
        } else if state.drag_origin_vertical.is_some() || state.is_resize_hovered_vertical {
            mouse::Interaction::ResizingVertically
        } else if self.clickable {
            if cursor.is_over(layout.bounds()) {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            let mut style = style::Catalog::style(theme, self.styling);
            if let Some(color) = self.change_color {
                match self.styling {
                    style::Styling::Value(change) | style::Styling::RowAlternating(change, _) => {
                        if change {
                            let border = Border {
                                color,
                                ..style.border
                            };
                            style = style.border(border);
                        }
                    }
                    _ => {}
                }
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: Shadow::default(),
                    snap: false,
                },
                style.background,
            );
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                renderer_style,
                layout.children().next().unwrap(),
                cursor,
                &clipped_viewport,
            );
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Cell<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: style::Catalog + 'a,
{
    fn from(cell: Cell<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(cell)
    }
}
