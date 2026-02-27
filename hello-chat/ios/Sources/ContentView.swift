import SwiftUI

struct ContentView: View {
    @Bindable var manager: AppManager
    @State private var navPath: [Screen] = []

    var body: some View {
        NavigationStack(path: $navPath) {
            ConversationListView(manager: manager)
                .navigationDestination(for: Screen.self) { screen in
                    screenView(for: screen)
                }
        }
        .onChange(of: manager.state.router.screenStack) { _, new in
            navPath = new
        }
        .onChange(of: navPath) { old, new in
            guard new != manager.state.router.screenStack else { return }
            if new.count < old.count {
                manager.dispatch(.updateScreenStack(stack: new))
            }
        }
    }

    @ViewBuilder
    private func screenView(for screen: Screen) -> some View {
        switch screen {
        case .conversationList:
            ConversationListView(manager: manager)
        case .chat(let conversationId):
            ChatView(manager: manager, conversationId: conversationId)
        }
    }
}

// MARK: - Conversation List

struct ConversationListView: View {
    @Bindable var manager: AppManager

    var body: some View {
        List(manager.state.conversations, id: \.id) { conv in
            Button {
                manager.dispatch(.openConversation(conversationId: conv.id))
            } label: {
                ConversationRow(conversation: conv)
            }
        }
        .listStyle(.plain)
        .navigationTitle("Chats")
    }
}

struct ConversationRow: View {
    let conversation: Conversation

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(Color.blue)
                    .frame(width: 48, height: 48)
                Text(conversation.avatarLetter)
                    .font(.headline)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(conversation.name)
                        .font(.headline)
                    Spacer()
                    Text(conversation.timestamp)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                HStack {
                    Text(conversation.lastMessage)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                    Spacer()
                    if conversation.unreadCount > 0 {
                        Text("\(conversation.unreadCount)")
                            .font(.caption2.bold())
                            .foregroundStyle(.white)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.blue, in: Capsule())
                    }
                }
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Chat View

struct ChatView: View {
    @Bindable var manager: AppManager
    let conversationId: String
    @State private var composeText = ""

    var body: some View {
        VStack(spacing: 0) {
            messageList
            Divider()
            composeBar
        }
        .navigationTitle(manager.state.currentConversation?.name ?? "Chat")
        .navigationBarTitleDisplayMode(.inline)
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 8) {
                    if let detail = manager.state.currentConversation {
                        ForEach(detail.messages, id: \.id) { message in
                            MessageBubble(message: message)
                                .id(message.id)
                        }
                    }
                }
                .padding()
            }
            .onChange(of: manager.state.currentConversation?.messages.count) { _, _ in
                if let last = manager.state.currentConversation?.messages.last {
                    withAnimation {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    private var composeBar: some View {
        HStack(spacing: 8) {
            TextField("Type a message...", text: $composeText)
                .textFieldStyle(.roundedBorder)
                .onSubmit(sendMessage)

            Button(action: sendMessage) {
                Image(systemName: "arrow.up.circle.fill")
                    .font(.title2)
            }
            .disabled(composeText.trimmingCharacters(in: .whitespaces).isEmpty)
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    private func sendMessage() {
        let text = composeText.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        manager.dispatch(.sendMessage(conversationId: conversationId, content: text))
        composeText = ""
    }
}

// MARK: - Message Bubble

struct MessageBubble: View {
    let message: ChatMessage

    var body: some View {
        HStack {
            if message.isMine { Spacer(minLength: 60) }

            VStack(alignment: message.isMine ? .trailing : .leading, spacing: 4) {
                if !message.isMine {
                    Text(message.sender)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Text(message.content)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(
                        message.isMine ? Color.blue : Color(.systemGray5),
                        in: RoundedRectangle(cornerRadius: 16)
                    )
                    .foregroundStyle(message.isMine ? .white : .primary)

                Text(message.timestamp)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            if !message.isMine { Spacer(minLength: 60) }
        }
    }
}


