use yew::{html, Component, Context, Html};
use super::sidebar::Sidebar;

pub struct Container;

impl Component for Container {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class={"container"}>
                <Sidebar/>
            </div>
        }
    }
}
