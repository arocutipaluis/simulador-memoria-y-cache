mod app;
mod components;
mod sim;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
