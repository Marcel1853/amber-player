package de.amberplayer.mobile

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.PowerManager
import android.provider.Settings
import android.webkit.WebView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class UrlArgs {
    var url: String = ""
}

@InvokeArg
class FullscreenArgs {
    var on: Boolean = true
}

/** Was gerade läuft – kommt bei jeder Statusänderung von der Oberfläche. */
@InvokeArg
class MediaArgs {
    var title: String = ""
    var artist: String = ""
    var album: String = ""
    var playing: Boolean = false
    var positionMs: Long = 0
    var durationMs: Long = 0
    /** Cover als Base64-JPEG (klein), leer = kein Cover */
    var cover: String? = null
}

/**
 * Brücke zwischen der Oberfläche von Amber Player und Android:
 * Benachrichtigung/Sperrbildschirm, Medientasten, Berechtigungen.
 * Tastendrücke gehen als Ereignis „media“ zurück an die Oberfläche.
 */
@TauriPlugin(
    permissions = [
        Permission(strings = ["android.permission.READ_MEDIA_AUDIO"], alias = "audio"),
        Permission(strings = ["android.permission.POST_NOTIFICATIONS"], alias = "notifications"),
    ]
)
class AmberMobilePlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        /** Für den Hintergrunddienst, um Tastendrücke weiterzureichen. */
        var instance: AmberMobilePlugin? = null
    }

    /** Kopplungs-Link vom QR-Code, falls die App damit gestartet wurde. */
    private var pendingPair: String? = null

    override fun load(webView: WebView) {
        instance = this
        pairLinkOf(activity.intent)?.let { pendingPair = it }
    }

    override fun onNewIntent(intent: Intent) {
        val link = pairLinkOf(intent) ?: return
        pendingPair = link
        val payload = JSObject()
        payload.put("link", link)
        trigger("pair", payload)
    }

    private fun pairLinkOf(intent: Intent?): String? {
        val data = intent?.data ?: return null
        return if (data.scheme == "amberplayer" && data.host == "pair") data.toString() else null
    }

    /** Liefert (einmalig) den Kopplungs-Link, mit dem die App gestartet wurde. */
    @Command
    fun takePairLink(invoke: Invoke) {
        val result = JSObject()
        result.put("link", pendingPair)
        pendingPair = null
        invoke.resolve(result)
    }

    /** Ereignis an die Oberfläche: play, pause, next, prev, stop, seek. */
    fun emit(action: String, positionMs: Long = 0) {
        val payload = JSObject()
        payload.put("action", action)
        payload.put("positionMs", positionMs)
        trigger("media", payload)
    }

    @Command
    fun updateMedia(invoke: Invoke) {
        val args = invoke.parseArgs(MediaArgs::class.java)
        PlaybackService.update(activity, args)
        invoke.resolve()
    }

    @Command
    fun stopMedia(invoke: Invoke) {
        PlaybackService.stop(activity)
        invoke.resolve()
    }

    /** Vollbild (Status- und Navigationsleiste ausgeblendet) – gleiche Einstellung wie in MainActivity. */
    @Command
    fun getFullscreen(invoke: Invoke) {
        val result = JSObject()
        result.put("on", activity.getSharedPreferences("amber_prefs", Activity.MODE_PRIVATE).getBoolean("fullscreen", true))
        invoke.resolve(result)
    }

    @Command
    fun setFullscreen(invoke: Invoke) {
        val on = invoke.parseArgs(FullscreenArgs::class.java).on
        activity.getSharedPreferences("amber_prefs", Activity.MODE_PRIVATE).edit().putBoolean("fullscreen", on).apply()
        activity.runOnUiThread {
            val controller = WindowCompat.getInsetsController(activity.window, activity.window.decorView)
            if (on) {
                controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                controller.hide(WindowInsetsCompat.Type.systemBars())
            } else {
                controller.show(WindowInsetsCompat.Type.systemBars())
            }
        }
        invoke.resolve()
    }

    /** Download-Link eines Updates im Browser öffnen (nur Adressen von Amber Player auf GitHub). */
    @Command
    fun openUrl(invoke: Invoke) {
        val url = invoke.parseArgs(UrlArgs::class.java).url
        if (!url.startsWith("https://github.com/Marcel1853/amber-player/")) {
            invoke.reject("Nur Adressen von Amber Player auf GitHub")
            return
        }
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(intent)
        invoke.resolve()
    }

    /** Fragt, ob Amber Player von der Akkuoptimierung ausgenommen werden darf (sonst stoppt Samsung die Musik). */
    @Command
    fun ignoreBatteryOptimizations(invoke: Invoke) {
        val pm = activity.getSystemService(PowerManager::class.java)
        val result = JSObject()
        if (pm != null && pm.isIgnoringBatteryOptimizations(activity.packageName)) {
            result.put("already", true)
        } else {
            val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, Uri.parse("package:" + activity.packageName))
            activity.startActivity(intent)
            result.put("already", false)
        }
        invoke.resolve(result)
    }
}
