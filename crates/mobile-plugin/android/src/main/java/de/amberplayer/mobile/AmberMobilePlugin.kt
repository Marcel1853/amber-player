package de.amberplayer.mobile

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.PowerManager
import android.provider.OpenableColumns
import android.provider.Settings
import android.util.Base64
import androidx.core.content.FileProvider
import java.io.File
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
class ShareArgs {
    var path: String = ""
    var title: String = ""
}

@InvokeArg
class FullscreenArgs {
    var on: Boolean = true
    /** false: nur vorübergehend (z. B. Visualisierung), die Einstellung bleibt unverändert */
    var remember: Boolean = true
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

    /** Geteilte Plugin-Datei (Name + Inhalt in Base64), falls die App damit geöffnet wurde. */
    private var pendingFile: JSObject? = null

    override fun load(webView: WebView) {
        instance = this
        pairLinkOf(activity.intent)?.let { pendingPair = it }
        sharedFileOf(activity.intent)?.let { pendingFile = it }
    }

    override fun onNewIntent(intent: Intent) {
        sharedFileOf(intent)?.let {
            pendingFile = it
            trigger("file", it)
            return
        }
        val link = pairLinkOf(intent) ?: return
        pendingPair = link
        val payload = JSObject()
        payload.put("link", link)
        trigger("pair", payload)
    }

    /**
     * Datei, die per „Öffnen mit“ oder „Teilen“ an Amber Player ging (z. B. über Quick Share).
     * Nur Plugin-Pakete bis 8 MB; ob es wirklich eins ist, prüft die Oberfläche beim Entpacken.
     */
    private fun sharedFileOf(intent: Intent?): JSObject? {
        if (intent == null) return null
        val uri: Uri = when (intent.action) {
            Intent.ACTION_VIEW -> intent.data
            Intent.ACTION_SEND -> @Suppress("DEPRECATION") (intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM))
            else -> null
        } ?: return null
        if (uri.scheme != "content" && uri.scheme != "file") return null
        return try {
            var name = uri.lastPathSegment ?: "plugin.amberplugin"
            activity.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
                if (c.moveToFirst()) c.getString(0)?.let { name = it }
            }
            val bytes = activity.contentResolver.openInputStream(uri)?.use { input ->
                val buf = java.io.ByteArrayOutputStream()
                val chunk = ByteArray(64 * 1024)
                while (true) {
                    val n = input.read(chunk)
                    if (n < 0) break
                    buf.write(chunk, 0, n)
                    if (buf.size() > 8 * 1024 * 1024) return null
                }
                buf.toByteArray()
            } ?: return null
            // Nur ZIP-Dateien (Plugin-Pakete beginnen mit „PK“)
            if (bytes.size < 4 || bytes[0] != 'P'.code.toByte() || bytes[1] != 'K'.code.toByte()) return null
            val obj = JSObject()
            obj.put("name", name)
            obj.put("data", Base64.encodeToString(bytes, Base64.NO_WRAP))
            obj
        } catch (e: Exception) {
            null
        }
    }

    /** Liefert (einmalig) die geteilte Plugin-Datei, mit der die App geöffnet wurde. */
    @Command
    fun takeSharedFile(invoke: Invoke) {
        val result = JSObject()
        result.put("file", pendingFile)
        pendingFile = null
        invoke.resolve(result)
    }

    /** Android-Teilen-Menü für eine Datei aus dem Cache-Ordner (Quick Share, Bluetooth, Messenger …). */
    @Command
    fun shareFile(invoke: Invoke) {
        val args = invoke.parseArgs(ShareArgs::class.java)
        val file = File(args.path)
        if (!file.exists() || !file.canonicalPath.startsWith(activity.cacheDir.canonicalPath)) {
            invoke.reject("Datei nicht gefunden")
            return
        }
        val uri = FileProvider.getUriForFile(activity, activity.packageName + ".fileprovider", file)
        val send = Intent(Intent.ACTION_SEND).apply {
            type = "application/octet-stream"
            putExtra(Intent.EXTRA_STREAM, uri)
            putExtra(Intent.EXTRA_SUBJECT, args.title)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        val chooser = Intent.createChooser(send, args.title).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        activity.startActivity(chooser)
        invoke.resolve()
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
        val args = invoke.parseArgs(FullscreenArgs::class.java)
        val on = args.on
        if (args.remember) {
            activity.getSharedPreferences("amber_prefs", Activity.MODE_PRIVATE).edit().putBoolean("fullscreen", on).apply()
        }
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

    /** Zurück-Taste auf der Startseite: App in den Hintergrund schicken statt beenden – die Musik läuft weiter. */
    @Command
    fun moveToBack(invoke: Invoke) {
        activity.runOnUiThread { activity.moveTaskToBack(true) }
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
