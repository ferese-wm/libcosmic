// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Builder;
use crate::Element;
use crate::widget::image::Handle;
use crate::widget::{self};
use iced_core::font::Weight;
use iced_core::widget::Id;
use iced_core::{Length, Padding};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Image<'a, Handle, Message>>;

/// A button constructed from an image handle, using image button styling.
pub fn image<'a, Message>(handle: impl Into<Handle> + 'a) -> Button<'a, Message> {
    Button::new(Image {
        image: widget::image(handle),
        selected: false,
        on_remove: None,
    })
}

/// The image variant of a button.
pub struct Image<'a, Handle, Message> {
    image: widget::Image<'a, Handle>,
    selected: bool,
    on_remove: Option<Message>,
}

impl<'a, Message> Button<'a, Message> {
    #[inline]
    pub fn new(variant: Image<'a, Handle, Message>) -> Self {
        Self {
            id: Id::unique(),
            label: Cow::Borrowed(""),
            #[cfg(feature = "a11y")]
            name: Cow::Borrowed(""),
            #[cfg(feature = "a11y")]
            description: Cow::Borrowed(""),
            tooltip: Cow::Borrowed(""),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::from(0),
            spacing: 0,
            icon_size: 16,
            line_height: 20,
            font_size: 14,
            font_weight: Weight::Normal,
            class: crate::theme::style::Button::Image,
            variant,
        }
    }

    #[inline]
    pub fn on_remove(mut self, message: Message) -> Self {
        self.variant.on_remove = Some(message);
        self
    }

    #[inline]
    pub fn on_remove_maybe(mut self, message: Option<Message>) -> Self {
        self.variant.on_remove = message;
        self
    }

    #[inline]
    pub fn selected(mut self, selected: bool) -> Self {
        self.variant.selected = selected;
        self
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Handle: Clone + std::hash::Hash,
    Message: Clone + 'static,
{
    fn from(builder: Button<'a, Message>) -> Element<'a, Message> {
        let content = builder
            .variant
            .image
            .width(builder.width)
            .height(builder.height);

        let mut button = super::custom_image_button(content, builder.variant.on_remove)
            .padding(0)
            .selected(builder.variant.selected)
            .id(builder.id)
            .on_press_maybe(builder.on_press)
            .class(builder.class);

        #[cfg(feature = "a11y")]
        {
            button = button.name(builder.name).description(builder.description);
        }

        button.into()
    }
}

#[cfg(all(test, feature = "wgpu", feature = "tokio"))]
mod tests {
    use super::*;
    use crate::iced::advanced::renderer::{Headless, Renderer as _};
    use crate::iced::advanced::{Layout, layout, mouse, renderer, widget::Tree};
    use crate::iced::border::{Outline, Shape};
    use crate::iced::{Color, Font, Pixels, Rectangle, Size};

    #[test]
    fn image_button_content_matches_its_theme_contour() {
        check("tiny-skia");
    }

    #[test]
    #[ignore = "requires a GPU or software Vulkan adapter"]
    fn gpu_image_button_content_matches_its_theme_contour() {
        check("wgpu");
    }

    fn check(backend: &str) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut renderer = runtime
            .block_on(<crate::Renderer as Headless>::new(
                Font::default(),
                Pixels(14.0),
                Some(backend),
            ))
            .unwrap();
        let bounds = Rectangle::with_size(Size::new(100.0, 100.0));

        for shape in [Shape::Circular, Shape::Continuous] {
            let theme = crate::Theme::dark().corner_shape(shape);
            let handle = crate::iced::widget::image::Handle::from_rgba(100, 100, vec![255; 40000]);
            let mut view: crate::Element<'_, ()> = image(handle)
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(100.0))
                .on_press(())
                .into();
            let mut tree = Tree::new(view.as_widget());
            let node = view.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, bounds.size()),
            );
            let outline = Outline::new(
                [0.0, 0.0, 100.0, 100.0],
                theme.cosmic().radius_s().map(f64::from),
                shape,
            )
            .unwrap();

            for scale in [1.0f32, 1.25, 1.5] {
                renderer.reset(bounds);
                view.as_widget().draw(
                    &tree,
                    &mut renderer,
                    &theme,
                    &renderer::Style {
                        text_color: Color::WHITE,
                        icon_color: Color::WHITE,
                        scale_factor: scale as f64,
                    },
                    Layout::new(&node),
                    mouse::Cursor::Unavailable,
                    &bounds,
                );
                let size = Size::new((100.0 * scale) as u32, (100.0 * scale) as u32);
                let pixels = Headless::screenshot(&mut renderer, size, scale, Color::TRANSPARENT);
                let reference = outline.transformed([0.0; 2], scale as f64).unwrap();

                for y in 0..size.height {
                    for x in 0..size.width {
                        let d = reference.signed_distance([x as f64 + 0.5, y as f64 + 0.5]);
                        let expected = (0.5 - d).clamp(0.0, 1.0);
                        let actual = pixels[((y * size.width + x) * 4 + 3) as usize] as f64 / 255.0;
                        assert!(
                            (actual - expected).abs() <= 2.0 / 255.0,
                            "{backend} {shape:?} scale={scale} pixel=({x},{y}) alpha={actual} reference={expected}"
                        );
                    }
                }
            }
        }
    }
}
