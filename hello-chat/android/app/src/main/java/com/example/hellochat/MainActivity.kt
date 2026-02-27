package com.example.hellochat

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.example.hellochat.ui.MainApp
import com.example.hellochat.ui.theme.AppTheme

class MainActivity : ComponentActivity() {
    private lateinit var manager: AppManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        manager = AppManager.getInstance(applicationContext)
        setContent {
            AppTheme {
                MainApp(manager = manager)
            }
        }
    }
}
