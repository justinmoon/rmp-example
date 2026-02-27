use iced::widget::{button, center, column, container, row, scrollable, text, text_input};
use iced::{color, Border, Element, Fill, Subscription, Task, Theme};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use hello_chat_core::{
    AppAction, AppReconciler, AppState, AppUpdate, ChatMessage, Conversation, ConversationDetail,
    FfiApp,
};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("Hello Chat")
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}

// ── AppManager ──────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppManager {
    ffi: Arc<FfiApp>,
    update_rx: flume::Receiver<()>,
}

impl Hash for AppManager {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.ffi).hash(state);
    }
}

impl AppManager {
    fn new() -> Result<Self, String> {
        let data_dir = dirs_next::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("HelloChat")
            .to_string_lossy()
            .to_string();
        let _ = std::fs::create_dir_all(&data_dir);

        let ffi = FfiApp::new(data_dir);
        let (notify_tx, update_rx) = flume::unbounded();
        ffi.listen_for_updates(Box::new(DesktopReconciler { tx: notify_tx }));

        Ok(Self { ffi, update_rx })
    }

    fn state(&self) -> AppState {
        self.ffi.state()
    }

    fn dispatch(&self, action: AppAction) {
        self.ffi.dispatch(action);
    }

    fn subscribe_updates(&self) -> flume::Receiver<()> {
        self.update_rx.clone()
    }
}

struct DesktopReconciler {
    tx: flume::Sender<()>,
}

impl AppReconciler for DesktopReconciler {
    fn reconcile(&self, _update: AppUpdate) {
        let _ = self.tx.send(());
    }
}

fn manager_update_stream(manager: &AppManager) -> impl iced::futures::Stream<Item = ()> {
    let rx = manager.subscribe_updates();
    iced::futures::stream::unfold(rx, |rx| async move {
        match rx.recv_async().await {
            Ok(()) => Some(((), rx)),
            Err(_) => None,
        }
    })
}

// ── App ─────────────────────────────────────────────────────────────────────

enum App {
    BootError {
        error: String,
    },
    Loaded {
        manager: AppManager,
        state: AppState,
        compose_input: String,
    },
}

#[derive(Debug, Clone)]
enum Message {
    CoreUpdated,
    OpenConversation(String),
    ComposeChanged(String),
    SendMessage,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = match AppManager::new() {
            Ok(manager) => {
                let state = manager.state();
                Self::Loaded {
                    manager,
                    state,
                    compose_input: String::new(),
                }
            }
            Err(error) => Self::BootError { error },
        };
        (app, Task::none())
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            App::BootError { .. } => Subscription::none(),
            App::Loaded { manager, .. } => {
                Subscription::run_with(manager.clone(), manager_update_stream)
                    .map(|_| Message::CoreUpdated)
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            App::BootError { .. } => {}
            App::Loaded {
                manager,
                state,
                compose_input,
            } => match message {
                Message::CoreUpdated => {
                    let latest = manager.state();
                    if latest.rev > state.rev {
                        *compose_input = latest.compose_text.clone();
                        *state = latest;
                    }
                }
                Message::OpenConversation(id) => {
                    manager.dispatch(AppAction::OpenConversation {
                        conversation_id: id,
                    });
                }
                Message::ComposeChanged(text) => {
                    *compose_input = text;
                }
                Message::SendMessage => {
                    if let Some(ref detail) = state.current_conversation {
                        let content = compose_input.trim().to_string();
                        if !content.is_empty() {
                            manager.dispatch(AppAction::SendMessage {
                                conversation_id: detail.conversation_id.clone(),
                                content,
                            });
                            compose_input.clear();
                        }
                    }
                }
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        match self {
            App::BootError { error } => center(
                column![
                    text("Hello Chat").size(24),
                    text(error).color(color!(0xCC3333)),
                ]
                .spacing(12),
            )
            .into(),

            App::Loaded {
                state,
                compose_input,
                ..
            } => {
                let sidebar =
                    view_sidebar(&state.conversations, state.current_conversation.as_ref());
                let detail = view_detail(state.current_conversation.as_ref(), compose_input);

                row![
                    container(sidebar).width(300).height(Fill),
                    container(column![]).width(1).height(Fill).style(|_: &Theme| container::Style {
                        background: Some(iced::Background::Color(color!(0x444444))),
                        ..Default::default()
                    }),
                    container(detail).width(Fill).height(Fill),
                ]
                .into()
            }
        }
    }
}

// ── Sidebar (Conversation List) ─────────────────────────────────────────────

fn view_sidebar<'a>(
    conversations: &'a [Conversation],
    current: Option<&'a ConversationDetail>,
) -> Element<'a, Message> {
    let header = container(text("Chats").size(20)).padding([16, 16]);

    let items: Vec<Element<'a, Message>> = conversations
        .iter()
        .map(|conv| view_conversation_item(conv, current))
        .collect();

    let list = scrollable(column(items)).height(Fill);

    column![header, list].into()
}

fn view_conversation_item<'a>(
    conv: &'a Conversation,
    current: Option<&'a ConversationDetail>,
) -> Element<'a, Message> {
    let is_selected = current
        .map(|d| d.conversation_id == conv.id)
        .unwrap_or(false);

    let avatar = container(center(
        text(&conv.avatar_letter).size(16).color(color!(0xFFFFFF)),
    ))
    .width(40)
    .height(40)
    .style(move |_: &Theme| container::Style {
        background: Some(iced::Background::Color(color!(0x5577CC))),
        border: Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let name_row = if conv.unread_count > 0 {
        row![
            text(&conv.name).size(14),
            iced::widget::Space::new().width(Fill),
            container(
                text(format!("{}", conv.unread_count))
                    .size(11)
                    .color(color!(0xFFFFFF)),
            )
            .padding([2, 6])
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(color!(0x3388FF))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
    } else {
        row![text(&conv.name).size(14), iced::widget::Space::new().width(Fill),]
    };

    let preview = row![
        text(&conv.last_message)
            .size(12)
            .color(color!(0x888888)),
        iced::widget::Space::new().width(Fill),
        text(&conv.timestamp).size(11).color(color!(0x666666)),
    ];

    let content = row![
        avatar,
        column![name_row, preview].spacing(4).width(Fill),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    let bg = if is_selected {
        Some(iced::Background::Color(color!(0x333344)))
    } else {
        None
    };

    button(container(content).padding([10, 12]))
        .on_press(Message::OpenConversation(conv.id.clone()))
        .width(Fill)
        .style(move |_, _| button::Style {
            background: bg,
            text_color: color!(0xDDDDDD),
            border: Border::default(),
            ..Default::default()
        })
        .into()
}

// ── Detail (Chat View) ─────────────────────────────────────────────────────

fn view_detail<'a>(
    detail: Option<&'a ConversationDetail>,
    compose_input: &'a str,
) -> Element<'a, Message> {
    match detail {
        None => center(
            text("Select a conversation")
                .size(16)
                .color(color!(0x666666)),
        )
        .into(),

        Some(detail) => {
            let header = container(
                row![text(&detail.name).size(18),].align_y(iced::Alignment::Center),
            )
            .padding([12, 16])
            .width(Fill);

            let messages: Vec<Element<'a, Message>> =
                detail.messages.iter().map(view_message).collect();

            let message_list =
                scrollable(column(messages).spacing(8).padding([8, 16]))
                    .height(Fill)
                    .anchor_bottom();

            let compose = container(
                row![
                    text_input("Type a message...", compose_input)
                        .on_input(Message::ComposeChanged)
                        .on_submit(Message::SendMessage)
                        .width(Fill),
                    button(text("Send").size(14))
                        .on_press(Message::SendMessage)
                        .padding([8, 16]),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding([8, 16]);

            column![header, message_list, compose].into()
        }
    }
}

fn view_message(msg: &ChatMessage) -> Element<'_, Message> {
    let bubble_color = if msg.is_mine {
        color!(0x3366AA)
    } else {
        color!(0x3A3A3A)
    };

    let sender_label = if !msg.is_mine {
        Some(text(&msg.sender).size(11).color(color!(0x88AADD)))
    } else {
        None
    };

    let content = text(&msg.content).size(14);
    let time = text(&msg.timestamp).size(10).color(color!(0x999999));

    let mut bubble_content = column![].spacing(4);
    if let Some(label) = sender_label {
        bubble_content = bubble_content.push(label);
    }
    bubble_content = bubble_content.push(content).push(time);

    let bubble = container(bubble_content)
        .padding([8, 12])
        .max_width(400)
        .style(move |_: &Theme| container::Style {
            background: Some(iced::Background::Color(bubble_color)),
            border: Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    if msg.is_mine {
        row![iced::widget::Space::new().width(Fill), bubble].width(Fill).into()
    } else {
        row![bubble, iced::widget::Space::new().width(Fill)].width(Fill).into()
    }
}
