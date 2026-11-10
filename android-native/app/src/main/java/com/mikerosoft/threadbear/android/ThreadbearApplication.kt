package com.mikerosoft.threadbear.android

import android.app.Application
import kotlinx.coroutines.MainScope

class ThreadbearApplication : Application() {
    lateinit var model: AppModel
        private set

    override fun onCreate() {
        super.onCreate()
        model = AppModel(this, MainScope())
    }
}
