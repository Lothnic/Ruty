//! Search bar widget (placeholder)

use iced::widget::{text_input, container};
use iced::{Element, Length};

/// Search bar component
pub fn view<'a, Message: Clone + 'a>(
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    text_input("Ask Ruty anything...", value)
        .on_input(on_input)
        .on_submit(on_submit)
        .padding(15)
        .size(18)
        .into()
}
