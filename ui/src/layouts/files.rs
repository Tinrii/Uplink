use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_router::*;
use kit::{
    components::nav::Nav,
    elements::{
        button::Button,
        file::File,
        folder::Folder,
        tooltip::{ArrowPosition, Tooltip},
        Appearance,
    },
    icons::{Icon, IconElement},
    layout::topbar::Topbar,
};
use shared::language::get_local_text;

use crate::{
    components::chat::{sidebar::Sidebar as ChatSidebar, RouteInfo},
};

#[derive(PartialEq, Props)]
pub struct Props {
    route_info: RouteInfo,
}

#[allow(non_snake_case)]
pub fn FilesLayout(cx: Scope<Props>) -> Element {
    let home_text = get_local_text("uplink.home");
    let free_space_text = get_local_text("files.free-space");
    let total_space_text = get_local_text("files.total-space");

    let desktop = use_window(cx);

    cx.render(rsx!(
        div {
            id: "files-layout",
            span {
                class: "hide-on-mobile",
                ChatSidebar {
                    route_info: cx.props.route_info.clone()
                },
            }
            div {
                class: "files-body",
                div {
                    onmousedown: move |_| { desktop.drag(); },
                    Topbar {
                        with_back_button: false,
                        controls: cx.render(
                            rsx! (
                                Button {
                                    icon: Icon::FolderPlus,
                                    appearance: Appearance::Secondary,
                                    tooltip: cx.render(rsx!(
                                        Tooltip {
                                            arrow_position: ArrowPosition::Top,
                                            text: get_local_text("files.new-folder"),
                                        }
                                    )),
                                    onpress: move |_| {
                                        // ...
                                    }
                                },
                                Button {
                                    icon: Icon::Plus,
                                    appearance: Appearance::Secondary,
                                    tooltip: cx.render(rsx!(
                                        Tooltip {
                                            arrow_position: ArrowPosition::Top,
                                            text: get_local_text("files.upload"),
                                        }
                                    ))
                                }
                            )
                        ),
                        div {
                            class: "files-info",
                            p {
                                class: "free-space",
                                "{free_space_text}",
                                span {
                                    class: "count",
                                    "0MB"
                                }
                            },
                            p {
                                class: "total-space",
                                "{total_space_text}",
                                span {
                                    class: "count",
                                    "10MB"
                                }
                            }
                        }
                    }
                },
                div {
                    class: "files-bar-track",
                    div {
                        class: "files-bar"
                    }
                },
                div {
                    class: "files-breadcrumbs",
                    div {
                        class: "crumb",
                        IconElement {
                            icon: Icon::Home,
                        },
                        p {
                            "{home_text}",
                        }
                    },
                    div {
                        class: "crumb",
                        p {
                            "Folder 1"
                        }
                    },
                    div {
                        class: "crumb",
                        p {
                            "Folder 3"
                        }
                    },
                },
                div {
                    class: "files-list",
                    span {
                        Folder {
                            text: "Fake Folder 1".into()
                        },
                        File {
                            text: "fake_2.png".into()
                        },
                        Folder {
                            text: "New Fake".into(),
                        },
                        Folder {
                            loading: true,
                            text: "Fake Folder 1".into()
                        },
                        File {
                            loading: true,
                            text: "Fake File".into()
                        }
                    }
                },
                span {
                    class: "hide-on-desktop",
                    Nav {
                        routes: cx.props.route_info.routes.clone(),
                        active: cx.props.route_info.active.clone(),
                        onnavigate: move |r| {
                            use_router(cx).replace_route(r, None, None);
                        }
                    }
                }
            }
        }
    ))
}
