use crate::Route;
use web_sys::{FormData, HtmlFormElement};
use yew::{html::Scope, html, Callback, Component, Context, Html, TargetCast};
use yew_router::prelude::*;

use super::{
    posts::PostQuery,
    template::{Body, TemplateMsg},
};

pub struct Sidebar {
    visibility: SidebarVisibility,
    style: String,
}

pub enum SidebarMsg {
    Toggle,
    Search(String),
}

pub enum SidebarVisibility {
    Show,
    Hidden,
}

impl Sidebar {
    fn generate_link(text: String, route: Route, query: Option<PostQuery>) -> Html {
        html! {
            <>
                <Link<Route, PostQuery>
                    { query }
                    to={route}
                >
                    <div class="indiv">
                        <div>
                            <a class="link" style="text-decoration: none;" >
                                    {text}
                            </a>
                        </div>
                    </div>
                </Link<Route, PostQuery>>
            </>
        }
    }

    fn links() -> Html {
        let query = PostQuery {
            sort: String::from("new"),
            ..Default::default()
        };

        html! {
            <>
                <center>
                    <div class="links">
                        { Self::generate_link(String::from("HOME"), Route::Home, None) }
                        { Self::generate_link(String::from("GALLERY"), Route::Gallery, Some(query)) }
                        { Self::generate_link(String::from("TAGS"), Route::Tags, None) }
                        { Self::generate_link(String::from("ABOUT"), Route::About, None) }
                        <div class="indiv">
                            <div>
                                <a href="https://github.com/player01osu/yuri-web"
                                    class="link"
                                    style="text-decoration: none;">
                                        {"GITHUB"}
                                </a>
                            </div>
                        </div>
                    </div>
                </center>
            </>
        }
    }

    fn cookie() -> Html {
        html! {
            <div class="nav-img">
                <div id="nav-wrap">
                    <img class="imge" src="../assets/img/blah.jpg" alt="nav-img"/>
                </div>
            </div>
        }
    }

    fn search_bar(link: &Scope<Self>) -> Html {
        let onsubmit = {
            link.callback(move |event: web_sys::FocusEvent| {
                event.prevent_default();
                let form = event.target_unchecked_into::<HtmlFormElement>();
                let data = FormData::new_with_form(&form).unwrap();

                let query = data.get("q").as_string().expect("Fucking parse this mf");
                SidebarMsg::Search(query)
            })
        };

        html! {
            <form
                action=""
                class="search-bar"
                {onsubmit}
            >
                <input
                    type="text"
                    class="search"
                    placeholder="search tag or somth"
                    name="q"/>
            </form>
        }
    }
}

impl Component for Sidebar {
    type Properties = ();
    type Message = SidebarMsg;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            visibility: SidebarVisibility::Show,
            style: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let (body, _) = ctx.link().context::<Body>(Callback::noop()).unwrap();
        match msg {
            SidebarMsg::Toggle => {
                match self.visibility {
                    SidebarVisibility::Show => {
                        self.style = "display: none !important;".to_string();
                        body.callback.emit(TemplateMsg::ToggleSidebar);
                        self.visibility = SidebarVisibility::Hidden;
                    }
                    SidebarVisibility::Hidden => {
                        self.style.clear();
                        body.callback.emit(TemplateMsg::ToggleSidebar);
                        self.visibility = SidebarVisibility::Show;
                    }
                }
                true
            }

            SidebarMsg::Search(query) => {
                let history = ctx.link().history().unwrap();
                let location = ctx.link().location().unwrap();

                let post_query = location.query::<PostQuery>().unwrap();
                let sort = post_query.sort;

                let query = PostQuery { sort, query };

                history.push_with_query(Route::Gallery, query).unwrap();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| SidebarMsg::Toggle);

        html! {
            <>
                <button class="hide-button"
                {onclick}> {"hide"} </button>

                <div class="navall" style={(&self.style).to_string()}>
                    <div class="nav">
                        { Self::search_bar(&ctx.link()) }
                        { Self::cookie() }
                        { Self::links() }
                    </div>
                </div>
            </>
        }
    }
}
