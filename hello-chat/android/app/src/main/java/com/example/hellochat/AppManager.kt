package com.example.hellochat

import android.content.Context
import android.os.Handler
import android.os.Looper
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.example.hellochat.rust.AppAction
import com.example.hellochat.rust.AppReconciler
import com.example.hellochat.rust.AppState
import com.example.hellochat.rust.AppUpdate
import com.example.hellochat.rust.Conversation
import com.example.hellochat.rust.FfiApp
import com.example.hellochat.rust.Router
import com.example.hellochat.rust.Screen

class AppManager private constructor(context: Context) : AppReconciler {
    private val mainHandler = Handler(Looper.getMainLooper())
    private val rust: FfiApp
    private var lastRevApplied: ULong = 0UL

    var state: AppState by mutableStateOf(
        AppState(
            rev = 0UL,
            router = Router(
                defaultScreen = Screen.ConversationList,
                screenStack = emptyList(),
            ),
            conversations = emptyList(),
            currentConversation = null,
            composeText = "",
            toast = null,
        ),
    )
        private set

    init {
        val dataDir = context.filesDir.absolutePath
        rust = FfiApp(dataDir)
        val initial = rust.state()
        state = initial
        lastRevApplied = initial.rev
        rust.listenForUpdates(this)
    }

    fun dispatch(action: AppAction) {
        rust.dispatch(action)
    }

    override fun reconcile(update: AppUpdate) {
        mainHandler.post {
            when (update) {
                is AppUpdate.FullState -> {
                    if (update.v1.rev <= lastRevApplied) return@post
                    lastRevApplied = update.v1.rev
                    state = update.v1
                }
            }
        }
    }

    companion object {
        @Volatile
        private var instance: AppManager? = null

        fun getInstance(context: Context): AppManager =
            instance ?: synchronized(this) {
                instance ?: AppManager(context.applicationContext).also { instance = it }
            }
    }
}
