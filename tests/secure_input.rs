use cosmic::iced::advanced::clipboard::Kind;
use cosmic::iced::advanced::{Clipboard, Widget, widget::Tree};
use cosmic::widget::text_context_menu::HasSelectableText;

struct MemoryClipboard(String);
impl Clipboard for MemoryClipboard {
    fn read(&self, _: Kind) -> Option<String> {
        Some(self.0.clone())
    }
    fn write(&mut self, _: Kind, value: String) {
        self.0 = value;
    }
}

#[test]
fn secure_menu_actions_do_not_export_or_cut_the_secret() {
    let mut input = cosmic::widget::text_input("Password", "dummy-secret-only")
        .password()
        .on_input(|_| ());
    let mut tree = Tree::new(&input as &dyn Widget<(), cosmic::Theme, cosmic::Renderer>);
    input.diff(&mut tree);
    input.select_all(&mut tree);
    let mut clipboard = MemoryClipboard("sentinel".into());
    // Native popup payload and overlay action both use this selection policy.
    assert_eq!(input.selected_text(&tree), None);
    input.copy_to_clipboard(&tree, &mut clipboard);
    assert_eq!(clipboard.0, "sentinel");
    assert_eq!(input.delete_selection(&mut tree), None);
    // Paste remains supported and replaces the selected password.
    assert_eq!(
        input.paste_text(&mut tree, "replacement"),
        Some("replacement".into())
    );
}

#[test]
fn ordinary_inputs_keep_copy_cut_and_paste() {
    let mut input = cosmic::widget::text_input("Name", "public text").on_input(|_| ());
    let mut tree = Tree::new(&input as &dyn Widget<(), cosmic::Theme, cosmic::Renderer>);
    input.diff(&mut tree);
    input.select_all(&mut tree);
    let mut clipboard = MemoryClipboard("sentinel".into());
    input.copy_to_clipboard(&tree, &mut clipboard);
    assert_eq!(clipboard.0, "public text");
    assert_eq!(input.delete_selection(&mut tree), Some(String::new()));
    assert_eq!(input.paste_text(&mut tree, "next"), Some("next".into()));
}

#[cfg(feature = "a11y")]
#[test]
fn accessible_password_has_a_name_and_actions_but_no_secret() {
    use cosmic::iced::advanced::Layout;
    use cosmic::iced::advanced::layout;
    use cosmic::iced::{Size, mouse};
    use iced_accessibility::accesskit::{Action, Role};
    let mut input = cosmic::widget::text_input("Password", "dummy-secret-only")
        .password()
        .label("Administrator password")
        .on_input(|_| ());
    let mut tree = Tree::new(&input as &dyn Widget<(), cosmic::Theme, cosmic::Renderer>);
    input.diff(&mut tree);
    input.select_all(&mut tree);
    let bounds = layout::Node::new(Size::new(300., 44.));
    let nodes = input.a11y_nodes(Layout::new(&bounds), &tree, mouse::Cursor::Unavailable);
    assert!(nodes.children().is_empty());
    let node = nodes.root()[0].node();
    assert_eq!(node.role(), Role::PasswordInput);
    assert_eq!(node.label(), Some("Administrator password"));
    assert_eq!(node.value(), None);
    assert!(node.supports_action(Action::Focus));
    assert!(node.supports_action(Action::SetValue));
    assert!(!format!("{nodes:?}").contains("dummy-secret-only"));
}

#[test]
fn keyboard_exports_are_blocked_and_escape_dismisses_menu_before_cancelling() {
    use cosmic::iced::advanced::{Layout, Shell, layout, renderer::Headless};
    use cosmic::iced::{Event, Font, Pixels, Rectangle, Size, keyboard, mouse};
    use cosmic::widget::text_input::State;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let renderer = runtime
        .block_on(<cosmic::Renderer as Headless>::new(
            Font::default(),
            Pixels(14.),
            Some("tiny-skia"),
        ))
        .unwrap();
    let mut input = cosmic::widget::text_input("Password", "dummy-secret-only")
        .password()
        .on_input(|_| 1)
        .on_escape(2);
    let mut tree = Tree::new(&input as &dyn Widget<i32, cosmic::Theme, cosmic::Renderer>);
    input.diff(&mut tree);
    tree.state.downcast_mut::<State>().focus();
    input.select_all(&mut tree);
    let node = input.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(300., 60.)),
    );
    let mut clipboard = MemoryClipboard("sentinel".into());
    for key in [
        keyboard::Key::Character("c".into()),
        keyboard::Key::Character("x".into()),
    ] {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyC),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::CTRL,
            text: None,
            repeat: false,
        });
        let mut messages = Vec::new();
        input.update(
            &mut tree,
            &event,
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(Size::new(300., 60.)),
        );
        assert_eq!(clipboard.0, "sentinel");
        assert_eq!(
            tree.state.downcast_ref::<State>().tracked_value.to_string(),
            "dummy-secret-only"
        );
    }
    input.set_context_menu_position(&mut tree, Some(cosmic::iced::Point::new(10., 10.)));
    let escape = Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Escape),
        modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    });
    for expected in [Vec::new(), vec![2]] {
        let mut messages = Vec::new();
        input.update(
            &mut tree,
            &escape,
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(Size::new(300., 60.)),
        );
        assert_eq!(messages, expected);
    }
}
