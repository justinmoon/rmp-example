use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

use flume::{Receiver, Sender};

uniffi::setup_scaffolding!();

// ── State ───────────────────────────────────────────────────────────────────

#[derive(uniffi::Record, Clone, Debug)]
pub struct AppState {
    pub rev: u64,
    pub router: Router,
    pub conversations: Vec<Conversation>,
    pub current_conversation: Option<ConversationDetail>,
    pub compose_text: String,
    pub toast: Option<String>,
}

#[derive(uniffi::Record, Clone, Debug, PartialEq)]
pub struct Router {
    pub default_screen: Screen,
    pub screen_stack: Vec<Screen>,
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum Screen {
    ConversationList,
    Chat { conversation_id: String },
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct Conversation {
    pub id: String,
    pub name: String,
    pub avatar_letter: String,
    pub last_message: String,
    pub timestamp: String,
    pub unread_count: u32,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ConversationDetail {
    pub conversation_id: String,
    pub name: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub is_mine: bool,
}

impl AppState {
    fn empty() -> Self {
        Self {
            rev: 0,
            router: Router {
                default_screen: Screen::ConversationList,
                screen_stack: vec![],
            },
            conversations: mock_conversations(),
            current_conversation: None,
            compose_text: String::new(),
            toast: None,
        }
    }
}

// ── Actions & Updates ───────────────────────────────────────────────────────

#[derive(uniffi::Enum, Clone, Debug)]
pub enum AppAction {
    PushScreen { screen: Screen },
    PopScreen,
    UpdateScreenStack { stack: Vec<Screen> },
    OpenConversation { conversation_id: String },
    SendMessage { conversation_id: String, content: String },
    UpdateComposeText { text: String },
    ClearToast,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum AppUpdate {
    FullState(AppState),
}

// ── Callback interface ──────────────────────────────────────────────────────

#[uniffi::export(callback_interface)]
pub trait AppReconciler: Send + Sync + 'static {
    fn reconcile(&self, update: AppUpdate);
}

// ── FFI entry point ─────────────────────────────────────────────────────────

enum CoreMsg {
    Action(AppAction),
}

#[derive(uniffi::Object)]
pub struct FfiApp {
    core_tx: Sender<CoreMsg>,
    update_rx: Receiver<AppUpdate>,
    listening: AtomicBool,
    shared_state: Arc<RwLock<AppState>>,
}

#[uniffi::export]
impl FfiApp {
    #[uniffi::constructor]
    pub fn new(data_dir: String) -> Arc<Self> {
        let _ = data_dir;

        let (update_tx, update_rx) = flume::unbounded();
        let (core_tx, core_rx) = flume::unbounded::<CoreMsg>();
        let shared_state = Arc::new(RwLock::new(AppState::empty()));

        let shared_for_core = shared_state.clone();
        thread::spawn(move || {
            let mut state = AppState::empty();
            let mut rev: u64 = 0;
            let mut next_msg_id: u64 = 100;

            let emit = |state: &AppState, shared: &Arc<RwLock<AppState>>, tx: &Sender<AppUpdate>| {
                let snapshot = state.clone();
                match shared.write() {
                    Ok(mut g) => *g = snapshot.clone(),
                    Err(p) => *p.into_inner() = snapshot.clone(),
                }
                let _ = tx.send(AppUpdate::FullState(snapshot));
            };

            emit(&state, &shared_for_core, &update_tx);

            while let Ok(msg) = core_rx.recv() {
                match msg {
                    CoreMsg::Action(action) => {
                        match action {
                            AppAction::PushScreen { screen } => {
                                state.router.screen_stack.push(screen);
                            }
                            AppAction::PopScreen => {
                                state.router.screen_stack.pop();
                                if !matches!(
                                    state.router.screen_stack.last(),
                                    Some(Screen::Chat { .. })
                                ) {
                                    state.current_conversation = None;
                                    state.compose_text.clear();
                                }
                            }
                            AppAction::UpdateScreenStack { stack } => {
                                state.router.screen_stack = stack;
                                if !matches!(
                                    state.router.screen_stack.last(),
                                    Some(Screen::Chat { .. })
                                ) {
                                    state.current_conversation = None;
                                    state.compose_text.clear();
                                }
                            }
                            AppAction::OpenConversation { conversation_id } => {
                                let detail = build_conversation_detail(&conversation_id);
                                state.current_conversation = Some(detail);
                                state.compose_text.clear();
                                if let Some(c) = state.conversations.iter_mut().find(|c| c.id == conversation_id) {
                                    c.unread_count = 0;
                                }
                                state.router.screen_stack.push(Screen::Chat {
                                    conversation_id,
                                });
                            }
                            AppAction::SendMessage { conversation_id, content } => {
                                let content = content.trim().to_string();
                                if content.is_empty() {
                                    continue;
                                }

                                next_msg_id += 1;
                                let new_msg = ChatMessage {
                                    id: format!("msg_{next_msg_id}"),
                                    sender: "You".to_string(),
                                    content: content.clone(),
                                    timestamp: "Just now".to_string(),
                                    is_mine: true,
                                };

                                if let Some(ref mut detail) = state.current_conversation {
                                    if detail.conversation_id == conversation_id {
                                        detail.messages.push(new_msg);
                                    }
                                }

                                if let Some(c) = state.conversations.iter_mut().find(|c| c.id == conversation_id) {
                                    c.last_message = format!("You: {content}");
                                    c.timestamp = "Just now".to_string();
                                }

                                state.compose_text.clear();
                            }
                            AppAction::UpdateComposeText { text } => {
                                state.compose_text = text;
                            }
                            AppAction::ClearToast => {
                                state.toast = None;
                            }
                        }

                        rev += 1;
                        state.rev = rev;
                        emit(&state, &shared_for_core, &update_tx);
                    }
                }
            }
        });

        Arc::new(Self {
            core_tx,
            update_rx,
            listening: AtomicBool::new(false),
            shared_state,
        })
    }

    pub fn state(&self) -> AppState {
        match self.shared_state.read() {
            Ok(g) => g.clone(),
            Err(poison) => poison.into_inner().clone(),
        }
    }

    pub fn dispatch(&self, action: AppAction) {
        let _ = self.core_tx.send(CoreMsg::Action(action));
    }

    pub fn listen_for_updates(&self, reconciler: Box<dyn AppReconciler>) {
        if self
            .listening
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let rx = self.update_rx.clone();
        thread::spawn(move || {
            while let Ok(update) = rx.recv() {
                reconciler.reconcile(update);
            }
        });
    }
}

// ── Mock Data ───────────────────────────────────────────────────────────────

fn mock_conversations() -> Vec<Conversation> {
    vec![
        Conversation {
            id: "conv_alice".to_string(),
            name: "Alice".to_string(),
            avatar_letter: "A".to_string(),
            last_message: "Hey! Are you coming to the party tonight?".to_string(),
            timestamp: "2:34 PM".to_string(),
            unread_count: 2,
        },
        Conversation {
            id: "conv_bob".to_string(),
            name: "Bob".to_string(),
            avatar_letter: "B".to_string(),
            last_message: "The build is passing now".to_string(),
            timestamp: "1:15 PM".to_string(),
            unread_count: 0,
        },
        Conversation {
            id: "conv_team".to_string(),
            name: "Team Chat".to_string(),
            avatar_letter: "T".to_string(),
            last_message: "Carol: Let's sync at 3".to_string(),
            timestamp: "12:45 PM".to_string(),
            unread_count: 5,
        },
        Conversation {
            id: "conv_dana".to_string(),
            name: "Dana".to_string(),
            avatar_letter: "D".to_string(),
            last_message: "Thanks for the review!".to_string(),
            timestamp: "Yesterday".to_string(),
            unread_count: 0,
        },
        Conversation {
            id: "conv_eve".to_string(),
            name: "Eve".to_string(),
            avatar_letter: "E".to_string(),
            last_message: "See you tomorrow".to_string(),
            timestamp: "Monday".to_string(),
            unread_count: 0,
        },
    ]
}

fn build_conversation_detail(conversation_id: &str) -> ConversationDetail {
    match conversation_id {
        "conv_alice" => ConversationDetail {
            conversation_id: "conv_alice".to_string(),
            name: "Alice".to_string(),
            messages: vec![
                ChatMessage {
                    id: "msg_1".to_string(),
                    sender: "Alice".to_string(),
                    content: "Hey! How's it going?".to_string(),
                    timestamp: "2:30 PM".to_string(),
                    is_mine: false,
                },
                ChatMessage {
                    id: "msg_2".to_string(),
                    sender: "You".to_string(),
                    content: "Pretty good! Working on the new project.".to_string(),
                    timestamp: "2:31 PM".to_string(),
                    is_mine: true,
                },
                ChatMessage {
                    id: "msg_3".to_string(),
                    sender: "Alice".to_string(),
                    content: "Nice! Are you coming to the party tonight?".to_string(),
                    timestamp: "2:33 PM".to_string(),
                    is_mine: false,
                },
                ChatMessage {
                    id: "msg_4".to_string(),
                    sender: "Alice".to_string(),
                    content: "It starts at 8pm at the usual place".to_string(),
                    timestamp: "2:34 PM".to_string(),
                    is_mine: false,
                },
            ],
        },
        "conv_bob" => ConversationDetail {
            conversation_id: "conv_bob".to_string(),
            name: "Bob".to_string(),
            messages: vec![
                ChatMessage {
                    id: "msg_10".to_string(),
                    sender: "You".to_string(),
                    content: "Did you fix the CI issue?".to_string(),
                    timestamp: "1:00 PM".to_string(),
                    is_mine: true,
                },
                ChatMessage {
                    id: "msg_11".to_string(),
                    sender: "Bob".to_string(),
                    content: "Yeah, it was a flaky test. I added a retry.".to_string(),
                    timestamp: "1:10 PM".to_string(),
                    is_mine: false,
                },
                ChatMessage {
                    id: "msg_12".to_string(),
                    sender: "You".to_string(),
                    content: "Great, thanks!".to_string(),
                    timestamp: "1:12 PM".to_string(),
                    is_mine: true,
                },
                ChatMessage {
                    id: "msg_13".to_string(),
                    sender: "Bob".to_string(),
                    content: "The build is passing now".to_string(),
                    timestamp: "1:15 PM".to_string(),
                    is_mine: false,
                },
            ],
        },
        "conv_team" => ConversationDetail {
            conversation_id: "conv_team".to_string(),
            name: "Team Chat".to_string(),
            messages: vec![
                ChatMessage {
                    id: "msg_20".to_string(),
                    sender: "Alice".to_string(),
                    content: "Morning everyone!".to_string(),
                    timestamp: "9:00 AM".to_string(),
                    is_mine: false,
                },
                ChatMessage {
                    id: "msg_21".to_string(),
                    sender: "Bob".to_string(),
                    content: "Hey! Ready for standup?".to_string(),
                    timestamp: "9:30 AM".to_string(),
                    is_mine: false,
                },
                ChatMessage {
                    id: "msg_22".to_string(),
                    sender: "You".to_string(),
                    content: "Yep, joining now".to_string(),
                    timestamp: "9:31 AM".to_string(),
                    is_mine: true,
                },
                ChatMessage {
                    id: "msg_23".to_string(),
                    sender: "Carol".to_string(),
                    content: "Let's sync at 3".to_string(),
                    timestamp: "12:45 PM".to_string(),
                    is_mine: false,
                },
            ],
        },
        _ => ConversationDetail {
            conversation_id: conversation_id.to_string(),
            name: "Unknown".to_string(),
            messages: vec![
                ChatMessage {
                    id: "msg_30".to_string(),
                    sender: "Unknown".to_string(),
                    content: "Hello!".to_string(),
                    timestamp: "12:00 PM".to_string(),
                    is_mine: false,
                },
            ],
        },
    }
}
