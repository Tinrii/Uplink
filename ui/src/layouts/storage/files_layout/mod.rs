#[allow(unused_imports)]
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use common::icons::outline::Shape as Icon;
use common::language::get_local_text;
use common::state::{ui, Action, State};
use common::upload_file_channel::CANCEL_FILE_UPLOADLISTENER;
use common::warp_runner::{RayGunCmd, WarpCmd};
use common::WARP_CMD_CH;
use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_router::prelude::use_navigator;
use futures::channel::oneshot;
use kit::elements::label::Label;
use kit::{
    elements::{
        button::Button,
        tooltip::{ArrowPosition, Tooltip},
        Appearance,
    },
    layout::topbar::Topbar,
};
use rfd::FileDialog;
use uuid::Uuid;
use warp::raygun::Location;

pub mod controller;
pub mod file_modal;

use crate::components::files::upload_progress_bar::UploadProgressBar;
use crate::components::paste_files_with_shortcut;
use crate::layouts::chats::ChatSidebar;
use crate::layouts::slimbar::SlimbarLayout;
use crate::layouts::storage::files_layout::file_modal::get_file_modal;
use crate::layouts::storage::send_files_layout::modal::SendFilesLayoutModal;
use crate::layouts::storage::send_files_layout::SendFilesStartLocation;
use crate::layouts::storage::shared_component::{FilesAndFolders, FilesBreadcumbs};

use self::controller::{StorageController, UploadFileController};

use super::functions::{self, ChanCmd, UseEvalFn};

#[allow(non_snake_case)]
pub fn FilesLayout(cx: Scope<'_>) -> Element<'_> {
    let state = use_shared_state::<State>(cx)?;
    state.write_silent().ui.current_layout = ui::Layout::Storage;
    let storage_controller = StorageController::new(cx, state);
    let upload_file_controller = UploadFileController::new(cx, state.clone());
    let window = use_window(cx);
    let files_in_queue_to_upload = upload_file_controller.files_in_queue_to_upload.clone();
    let files_been_uploaded = upload_file_controller.files_been_uploaded.clone();
    let send_files_from_storage = use_state(cx, || false);
    let _router = use_navigator(cx);
    let eval: &UseEvalFn = use_eval(cx);

    functions::use_allow_block_folder_nav(cx, &files_in_queue_to_upload);

    let ch: &Coroutine<ChanCmd> = functions::init_coroutine(cx, storage_controller);

    use_future(cx, (), |_| {
        to_owned![files_been_uploaded, files_in_queue_to_upload];
        async move {
            // Remove load progress bar if anythings goes wrong
            loop {
                if files_in_queue_to_upload.read().is_empty() && *files_been_uploaded.read() {
                    *files_been_uploaded.write() = false;
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    });

    functions::run_verifications_and_update_storage(
        state,
        storage_controller,
        upload_file_controller.files_in_queue_to_upload,
    );

    functions::get_items_from_current_directory(cx, ch);

    #[cfg(not(target_os = "macos"))]
    functions::allow_drag_event_for_non_macos_systems(
        cx,
        upload_file_controller.are_files_hovering_app,
    );
    functions::start_upload_file_listener(
        cx,
        window,
        state,
        storage_controller,
        upload_file_controller.clone(),
    );

    let tx_cancel_file_upload = CANCEL_FILE_UPLOADLISTENER.tx.clone();

    cx.render(rsx!(
        if state.read().ui.metadata.focused  {
            rsx!(paste_files_with_shortcut::PasteFilesShortcut {
                on_paste: move |files_local_path| {
                    functions::add_files_in_queue_to_upload(&files_in_queue_to_upload, files_local_path, eval);
                    upload_file_controller.files_been_uploaded.with_mut(|i| *i = true);
                },
            })
        }
        if let Some(file) = storage_controller.read().show_file_modal.as_ref() {
            let file2 = file.clone();
            rsx!(
                get_file_modal {
                    on_dismiss: |_| {
                        storage_controller.with_mut(|i| i.show_file_modal = None);
                    },
                    on_download: move |_| {
                        let file_name = file2.clone().name();
                        functions::download_file(&file_name, ch);
                    },
                    file: file.clone()
                }
            )
        }
        div {
            id: "files-layout",
            aria_label: "files-layout",
            ondragover: move |_| {
                if upload_file_controller.are_files_hovering_app.with(|i| !(i)) {
                    upload_file_controller.are_files_hovering_app.with_mut(|i| *i = true);
                }
                },
            onclick: |_| {
                storage_controller.write().finish_renaming_item(false);
            },
            SlimbarLayout {
                active: crate::UplinkRoute::FilesLayout {}
            },
            ChatSidebar {
                active_route: crate::UplinkRoute::FilesLayout {},
            },
            div {
                class: "files-body disable-select",
                aria_label: "files-body",
                    Topbar {
                        with_back_button: state.read().ui.is_minimal_view() && state.read().ui.sidebar_hidden,
                        onback: move |_| {
                            let current = state.read().ui.sidebar_hidden;
                            state.write().mutate(Action::SidebarHidden(!current));
                        },
                        controls: cx.render(
                            rsx! (Button {
                                    icon: Icon::FolderPlus,
                                    disabled: *upload_file_controller.files_been_uploaded.read(),
                                    appearance: Appearance::Secondary,
                                    aria_label: "add-folder".into(),
                                    tooltip: cx.render(rsx!(
                                        Tooltip {
                                            arrow_position: ArrowPosition::Top,
                                            text: get_local_text("files.new-folder"),
                                        }
                                    )),
                                    onpress: move |_| {
                                        if !*upload_file_controller.files_been_uploaded.read() {
                                            storage_controller.write().finish_renaming_item(true);
                                        }
                                    },
                                },
                                Button {
                                    icon: Icon::Plus,
                                    appearance: Appearance::Secondary,
                                    aria_label: "upload-file".into(),
                                    tooltip: cx.render(rsx!(
                                        Tooltip {
                                            arrow_position: ArrowPosition::Top,
                                            text: get_local_text("files.upload"),
                                        }
                                    )),
                                    onpress: move |_| {
                                        storage_controller.with_mut(|i|  i.is_renaming_map = None);
                                        let files_local_path = match FileDialog::new().set_directory(".").pick_files() {
                                            Some(path) => path,
                                            None => return
                                        };
                                        functions::add_files_in_queue_to_upload(upload_file_controller.files_in_queue_to_upload, files_local_path, eval);
                                        upload_file_controller.files_been_uploaded.with_mut(|i| *i = true);
                                    },
                                }
                            )
                        ),
                        div {
                            class: "files-info",
                            aria_label: "files-info",
                            if storage_controller.read().storage_size.0.is_empty() {
                                rsx!(div {
                                    class: "skeletal-texts",
                                    div {
                                        class: "skeletal-text",
                                        div {
                                            class: "skeletal-text-content skeletal",
                                        }
                                    },
                                },
                                div {
                                    class: "skeletal-texts",
                                    div {
                                        class: "skeletal-text",
                                        div {
                                            class: "skeletal-text-content skeletal",
                                        }
                                    },
                                })
                            } else {
                                rsx!(
                                    p {
                                        class: "free-space",
                                        aria_label: "free-space-max-size",
                                        get_local_text("files.storage-max-size"),
                                        span {
                                            class: "count",
                                            format!("{}", storage_controller.read().storage_size.0),
                                        }
                                    },
                                    p {
                                        class: "free-space",
                                        aria_label: "free-space-current-size",
                                        get_local_text("files.storage-current-size"),
                                        span {
                                            class: "count",
                                            format!("{}", storage_controller.read().storage_size.1),
                                        }
                                    },
                                )
                            }
                        }
                    }
                    UploadProgressBar {
                        are_files_hovering_app: upload_file_controller.are_files_hovering_app,
                        files_been_uploaded: upload_file_controller.files_been_uploaded,
                        disable_cancel_upload_button: upload_file_controller.disable_cancel_upload_button,
                        on_update: move |files_to_upload: Vec<PathBuf>|  {
                            functions::add_files_in_queue_to_upload(upload_file_controller.files_in_queue_to_upload, files_to_upload, eval);
                        },
                        on_cancel: move |_| {
                            let _ = tx_cancel_file_upload.send(true);
                            let _ = tx_cancel_file_upload.send(false);
                        },
                    },
            SendFilesLayoutModal {
                send_files_from_storage: send_files_from_storage,
                send_files_start_location: SendFilesStartLocation::Storage,
                on_send: move |(files_location, convs_id): (Vec<Location>, Vec<Uuid>)| {
                    let warp_cmd_tx = WARP_CMD_CH.tx.clone();
                    let (tx, _) = oneshot::channel::<Result<(), warp::error::Error>>();
                    let msg = vec!["".to_owned()];
                    let attachments = files_location;
                    let ui_msg_id = None;
                    let convs_id = convs_id;
                    if let Err(e) = warp_cmd_tx.send(WarpCmd::RayGun(RayGunCmd::SendMessageForSeveralChats {
                        convs_id,
                        msg,
                        attachments,
                        ui_msg_id,
                        rsp: tx,
                    })) {
                        log::error!("Failed to send warp command: {}", e);
                        return;
                    }
                    send_files_from_storage.set(false);
                }
            },
            FilesBreadcumbs {
                storage_controller: storage_controller,
                ch: ch,
                send_files_mode: false,
            },
            if storage_controller.read().files_list.is_empty()
                && storage_controller.read().directories_list.is_empty()
                && !storage_controller.read().add_new_folder {
                    rsx!(
                        div {
                            padding: "48px",
                            Label {
                                text: get_local_text("files.no-files-available"),
                            }
                        }
                        )
               } else {
                rsx!(FilesAndFolders {
                    storage_controller: storage_controller,
                    on_click_share_files: move |_| {
                        send_files_from_storage.set(true);
                    },
                    ch: ch,
                    send_files_mode: false,
                })
               }
                (state.read().ui.sidebar_hidden && state.read().ui.metadata.minimal_view).then(|| rsx!(
                    crate::AppNav {
                        active: crate::UplinkRoute::FilesLayout{},
                    }
                ))
            }
        }
    ))
}