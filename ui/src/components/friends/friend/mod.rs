use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use kit::{
    components::{
        indicator::{Platform, Status},
        user_image::UserImage,
    },
    elements::{
        button::Button,
        label::Label,
        tooltip::{ArrowPosition, Tooltip},
        Appearance,
    },
    icons::Icon,
};

use warp::multipass::identity::Relationship;

use crate::{
    state::State,
    utils::{format_timestamp::format_timestamp_timeago, language::get_local_text},
};

#[derive(Props)]
pub struct Props<'a> {
    // The username of the friend request sender
    username: String,
    // A suffix to the username, typically a unique identifier
    suffix: String,
    // Users relationship
    relationship: Relationship,
    // Time when request was sent or received
    #[props(optional)]
    request_datetime: Option<DateTime<Utc>>,
    // Status message from friend
    status_message: String,
    // The user image element to display
    user_image: Element<'a>,
    // An optional event handler for the "onchat" event
    #[props(optional)]
    onchat: Option<EventHandler<'a>>,
    // An optional event handler for the "onremove" event
    #[props(optional)]
    onremove: Option<EventHandler<'a>>,
    #[props(optional)]
    onaccept: Option<EventHandler<'a>>,
    // An optional event handler for the "onblock" event
    #[props(optional)]
    onblock: Option<EventHandler<'a>>,
}

#[allow(non_snake_case)]
pub fn Friend<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    let state: UseSharedState<State> = use_context::<State>(&cx).unwrap();
    let active_language = state.read().settings.language.clone();
    let relationship = cx.props.relationship;
    let status_message = cx.props.status_message.clone();
    let request_datetime = cx.props.request_datetime.unwrap_or_else(Utc::now);
    let formatted_timeago = format_timestamp_timeago(request_datetime, active_language);

    cx.render(rsx!(
        div {
            class: "friend",
            &cx.props.user_image,
            div {
                class: "request-info",
                p {
                    "{cx.props.username}",
                    span {
                        "#{cx.props.suffix}"
                    }
                },
                if relationship.friends() || relationship.blocked() {
                   rsx!(p {
                        class: "status-message",
                        "{status_message}"
                    })
                } else  {
                    rsx!(Label {
                        // TODO: this is stubbed for now, wire up to the actual request time
                        text: format!("{} {formatted_timeago}", 
                        if relationship.sent_friend_request() { get_local_text("friends.sent") } 
                        else { get_local_text("friends.requested") })
                    })
                }
            },
            div {
                class: "request-controls",
                cx.props.onaccept.is_some().then(|| rsx!(
                    Button {
                        icon: Icon::Check,
                        text: get_local_text("friends.accept"),
                        onpress: move |_| match &cx.props.onaccept {
                            Some(f) => f.call(()),
                            None    => {},
                        }
                    }
                )),
                cx.props.onchat.is_some().then(|| rsx! (
                    Button {
                        icon: Icon::ChatBubbleBottomCenterText,
                        text: get_local_text("uplink.chat"),
                        onpress: move |_| match &cx.props.onchat {
                            Some(f) => f.call(()),
                            None    => {},
                        }
                    }
                )),
                Button {
                    icon: Icon::UserMinus,
                    appearance: Appearance::Secondary,
                    onpress: move |_| match &cx.props.onremove {
                        Some(f) => f.call(()),
                        None    => {},
                    }
                    tooltip: cx.render(rsx!(
                        Tooltip {
                            arrow_position: ArrowPosition::Right,
                            text: if cx.props.onaccept.is_none() { get_local_text("friends.remove") } else { get_local_text("friends.deny") }
                        }
                    )),
                },
                cx.props.onchat.is_some().then(|| rsx!(
                    Button {
                        icon: Icon::NoSymbol,
                        appearance: Appearance::Secondary,
                        onpress: move |_| match &cx.props.onblock {
                            Some(f) => f.call(()),
                            None    => {},
                        }
                        tooltip: cx.render(rsx!(
                            Tooltip {
                                arrow_position: ArrowPosition::Right,
                                text: get_local_text("friends.block"),
                            }
                        )),
                    }
                ))
            }
        }
    ))
}

#[allow(non_snake_case)]
pub fn SkeletalFriend(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            class: "skeletal-friend",
            UserImage {
                loading: true,
                platform: Platform::Desktop,
                status: Status::Offline,
            },
            div {
                class: "skeletal-bars",
                div {
                    class: "skeletal-bar"
                },
                div {
                    class: "skeletal-bar"
                }
            },
            // TODO: include the disabled controls, should show only controls relevant to parent props.
        }
    ))
}
