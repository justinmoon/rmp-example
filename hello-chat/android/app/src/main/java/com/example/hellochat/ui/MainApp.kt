package com.example.hellochat.ui

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.hellochat.AppManager
import com.example.hellochat.rust.AppAction
import com.example.hellochat.rust.ChatMessage
import com.example.hellochat.rust.Conversation
import com.example.hellochat.rust.Screen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainApp(manager: AppManager) {
    val router = manager.state.router
    val current = router.screenStack.lastOrNull() ?: router.defaultScreen

    BackHandler(enabled = router.screenStack.isNotEmpty()) {
        manager.dispatch(
            AppAction.UpdateScreenStack(
                stack = router.screenStack.dropLast(1)
            )
        )
    }

    AnimatedContent(targetState = current, label = "screen") { screen ->
        when (screen) {
            is Screen.ConversationList -> ConversationListScreen(manager)
            is Screen.Chat -> ChatScreen(manager, screen.conversationId)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConversationListScreen(manager: AppManager) {
    Scaffold(
        topBar = {
            TopAppBar(title = { Text("Chats") })
        },
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
        ) {
            items(manager.state.conversations, key = { it.id }) { conv ->
                ConversationRow(
                    conversation = conv,
                    onClick = {
                        manager.dispatch(AppAction.OpenConversation(conversationId = conv.id))
                    },
                )
            }
        }
    }
}

@Composable
fun ConversationRow(conversation: Conversation, onClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(
            modifier = Modifier
                .size(48.dp)
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.primary),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                conversation.avatarLetter,
                color = MaterialTheme.colorScheme.onPrimary,
                fontWeight = FontWeight.Bold,
            )
        }

        Column(modifier = Modifier.weight(1f)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    conversation.name,
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    conversation.timestamp,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(modifier = Modifier.height(2.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    conversation.lastMessage,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    modifier = Modifier.weight(1f),
                )
                if (conversation.unreadCount > 0u) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Badge {
                        Text("${conversation.unreadCount}")
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatScreen(manager: AppManager, conversationId: String) {
    val detail = manager.state.currentConversation
    var composeText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()

    LaunchedEffect(detail?.messages?.size) {
        detail?.messages?.let {
            if (it.isNotEmpty()) {
                listState.animateScrollToItem(it.size - 1)
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(detail?.name ?: "Chat") },
                navigationIcon = {
                    IconButton(onClick = {
                        manager.dispatch(AppAction.PopScreen)
                    }) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                },
            )
        },
        bottomBar = {
            Surface(tonalElevation = 3.dp) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    OutlinedTextField(
                        value = composeText,
                        onValueChange = { composeText = it },
                        modifier = Modifier.weight(1f),
                        placeholder = { Text("Type a message...") },
                        singleLine = true,
                        shape = RoundedCornerShape(24.dp),
                    )
                    IconButton(
                        onClick = {
                            val text = composeText.trim()
                            if (text.isNotEmpty()) {
                                manager.dispatch(
                                    AppAction.SendMessage(
                                        conversationId = conversationId,
                                        content = text,
                                    )
                                )
                                composeText = ""
                            }
                        },
                        enabled = composeText.trim().isNotEmpty(),
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.Send,
                            contentDescription = "Send",
                            tint = if (composeText.trim().isNotEmpty())
                                MaterialTheme.colorScheme.primary
                            else
                                MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        },
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp),
            state = listState,
            verticalArrangement = Arrangement.spacedBy(8.dp),
            contentPadding = PaddingValues(vertical = 8.dp),
        ) {
            detail?.messages?.let { messages ->
                items(messages, key = { it.id }) { message ->
                    MessageBubble(message = message)
                }
            }
        }
    }
}

@Composable
fun MessageBubble(message: ChatMessage) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = if (message.isMine) Alignment.End else Alignment.Start,
    ) {
        if (!message.isMine) {
            Text(
                message.sender,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(start = 12.dp, bottom = 2.dp),
            )
        }
        Surface(
            shape = RoundedCornerShape(16.dp),
            color = if (message.isMine)
                MaterialTheme.colorScheme.primary
            else
                MaterialTheme.colorScheme.surfaceVariant,
        ) {
            Text(
                message.content,
                modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
                color = if (message.isMine)
                    MaterialTheme.colorScheme.onPrimary
                else
                    MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Text(
            message.timestamp,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(
                start = if (message.isMine) 0.dp else 12.dp,
                end = if (message.isMine) 12.dp else 0.dp,
                top = 2.dp,
            ),
        )
    }
}
