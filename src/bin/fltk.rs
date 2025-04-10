use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use fltk::button::CheckButton;
use fltk::group::{Flex, Group};
use fltk::menu::Choice;
use fltk::valuator::Slider;

fn main() {
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 600, "あほあほあぷり");
    let mut frame = Frame::new(0, 0, 400, 200, "あほ");
    let mut but = Button::new(160, 210, 80, 40, "あほボタン");
    let flex = Flex::default().with_size(400, 300).with_pos(0, 300).column();
    let mut aho = Frame::new(0, 0, 400, 200, "あなたのあほ度は0です。");
    let mut baka = Frame::new(0, 0, 400, 200, "ばか");
    let checkbox = CheckButton::new(0, 0, 200, 20, "あほチェックボックス");
    let mut choice = Choice::new(0, 0, 200, 20, "あほチョイス");
    choice.add_choice("あほ");
    choice.add_choice("ばか");
    choice.add_choice("おばかさん");
    choice.add_choice("おまぬけさん");
    flex.end();



    wind.end();
    wind.show();
    but.set_callback(move |_| frame.set_label("あほボタンを押したあなたはあほです。")); // the closure capture is mutable borrow to our button
    app.run().unwrap();
}