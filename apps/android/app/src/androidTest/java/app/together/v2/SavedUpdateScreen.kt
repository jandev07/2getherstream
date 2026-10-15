package app.together.v2

import android.app.Activity
import android.app.Application
import android.content.Context
import android.content.Intent
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.LocalSaveableStateRegistry
import androidx.compose.runtime.saveable.SaveableStateRegistry
import androidx.test.core.app.ActivityScenario
import app.together.v2.update.UpdateControls

/** Seed initial saved values only. Recreation and Settings return retain the real
 * Activity registry and production UpdateControls. No downloader/installer stub. */
internal class SavedUpdateScreen(context:Context,seed:List<Any>):AutoCloseable {
    private val context=context
    private val application=context.applicationContext as Application
    var creations=0
        private set
    private val callbacks=object:Application.ActivityLifecycleCallbacks {
        override fun onActivityCreated(a:Activity,b:Bundle?) {}
        override fun onActivityPostCreated(activity:Activity,state:Bundle?) {
            if(activity !is MainActivity)return
            val initial=creations++==0
            activity.setContent {
                val owner=checkNotNull(LocalSaveableStateRegistry.current)
                val registry=remember(owner) {
                    if(!initial)owner else object:SaveableStateRegistry by owner {
                        private val values=ArrayDeque(seed)
                        override fun consumeRestored(key:String):Any? =if(values.isEmpty())owner.consumeRestored(key) else values.removeFirst()
                    }
                }
                CompositionLocalProvider(LocalSaveableStateRegistry provides registry) {MaterialTheme {Column {UpdateControls()}}}
            }
        }
        override fun onActivityStarted(a:Activity) {}
        override fun onActivityResumed(a:Activity) {}
        override fun onActivityPaused(a:Activity) {}
        override fun onActivityStopped(a:Activity) {}
        override fun onActivitySaveInstanceState(a:Activity,b:Bundle) {}
        override fun onActivityDestroyed(a:Activity) {}
    }
    init {application.registerActivityLifecycleCallbacks(callbacks)}
    fun launch():ActivityScenario<MainActivity> =ActivityScenario.launch(Intent(context,MainActivity::class.java).putExtra("localProofOnly",true))
    override fun close() {application.unregisterActivityLifecycleCallbacks(callbacks)}
}
