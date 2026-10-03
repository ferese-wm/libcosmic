# Corner profiles

Built-in styles use `Theme::corner_shape`. It defaults to circular corners.
An application can select the shared squircle profile without changing radii:

```rust
let theme = cosmic::Theme::dark()
    .corner_shape(cosmic::iced::border::Shape::Continuous);
```

Standard Iced styles carry the profile in `Border`. COSMIC buttons, text inputs,
dropdowns, menus and tooltips expose a `shape` override in their style. A custom
COSMIC button or text-input style with `shape: None` uses the theme preference;
`Some(Shape::Circular)` explicitly keeps circular corners. Custom Iced styles
are returned unchanged, so their `Border` remains authoritative.

Segmented controls apply the profile to their status and item borders. Slider
rails and rectangular handles use it too; circular handles stay circular.
Progress indicator classes can override their shape independently. Pills,
circles and zero-radius corners keep their ordinary geometry.

## Reference insets

COSMIC button and text-input styles also accept an `outline`. It retains the
original bounds, radii and profile. Use `outline.inset(distance)` for a surface
that must follow an outer contour at a constant distance. Supply the reference
in the same logical coordinates as the widget's draw bounds. Renderer transforms
apply once. Outlined focus rings and shifted shadows retain that reference too.

This is explicit: nesting does not make child buttons concentric. Ordinary
children keep their own shape. Subtracting padding from a squircle radius and
rebuilding the child gives a different contour.

Iced's container exposes `clip_to_border(true)` and `clip_outline(outline)` for
child content that needs shaped clipping. Backgrounds alone do not clip children,
and clipping does not include separate overlays. Masking creates an intermediate
surface, so it is opt-in.

The shared crate documents the profile's approximation limits:
[ferese-shape](https://github.com/ferese-wm/ferese-shape). `Continuous` is the API
name for the measured squircle profile; its segment joins have small tangent and
curvature discontinuities.
