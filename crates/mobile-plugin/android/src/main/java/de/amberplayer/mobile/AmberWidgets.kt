package de.amberplayer.mobile

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.BroadcastReceiver
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.widget.RemoteViews
import java.io.File

/** Kompakt (4×1) */
class AmberWidgetSmall : AppWidgetProvider() {
    override fun onUpdate(ctx: Context, mgr: AppWidgetManager, ids: IntArray) = AmberWidgets.refresh(ctx)
}

/** Groß (4×2) */
class AmberWidgetLarge : AppWidgetProvider() {
    override fun onUpdate(ctx: Context, mgr: AppWidgetManager, ids: IntArray) = AmberWidgets.refresh(ctx)
}

/** Radio (4×2): laufender Sender und gespeicherte Sender zum Antippen */
class AmberWidgetRadio : AppWidgetProvider() {
    override fun onUpdate(ctx: Context, mgr: AppWidgetManager, ids: IntArray) = AmberWidgets.refresh(ctx)
}

/** Quadrat (2×2) */
class AmberWidgetSquare : AppWidgetProvider() {
    override fun onUpdate(ctx: Context, mgr: AppWidgetManager, ids: IntArray) = AmberWidgets.refresh(ctx)
}

/**
 * Tasten der Widgets. Läuft Amber Player, gehen sie wie die Medientasten direkt an den Player;
 * sonst wird die App geöffnet.
 */
class AmberWidgetAction : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        val action = intent.getStringExtra("action") ?: return
        val plugin = AmberMobilePlugin.instance
        if (plugin != null) {
            if (action.startsWith("station:")) plugin.emit("station", action.removePrefix("station:").toLongOrNull() ?: 0)
            else plugin.emit(action)
        } else {
            ctx.packageManager.getLaunchIntentForPackage(ctx.packageName)?.let {
                it.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                try {
                    ctx.startActivity(it)
                } catch (e: Exception) {
                    // Android erlaubt das Öffnen aus dem Hintergrund nicht immer – dann zeigen die
                    // Widgets „App öffnen“ an, der nächste Tipp öffnet sie direkt.
                    AmberWidgets.markStopped(ctx)
                }
            }
        }
    }
}

/** Zeichnet alle Widgets – aus dem letzten Stand, der auch nach Ende der App erhalten bleibt. */
object AmberWidgets {
    private const val PREFS = "amber_widget"
    private const val COVER = "widget_cover.png"

    /** Gespeicherte Sender mit Logos ablegen (für das Radio-Widget). */
    fun saveStations(ctx: Context, stations: List<StationItem>) {
        val ids = stations.joinToString(",") { it.id.toString() }
        val names = stations.joinToString("\u001F") { it.name }
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString("stationIds", ids).putString("stationNames", names).apply()
        for (s in stations) {
            val file = File(ctx.cacheDir, "widget_station_${s.id}.png")
            val bmp = decode(s.logo, 160)
            if (bmp != null) file.outputStream().use { bmp.compress(Bitmap.CompressFormat.PNG, 100, it) } else file.delete()
        }
        refresh(ctx)
    }

    /** Base64-Bild verkleinert laden. */
    private fun decode(data: String?, max: Int): Bitmap? {
        if (data.isNullOrEmpty()) return null
        return try {
            val bytes = android.util.Base64.decode(data, android.util.Base64.DEFAULT)
            val raw = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: return null
            if (raw.width > max || raw.height > max) Bitmap.createScaledBitmap(raw, max, max * raw.height / raw.width.coerceAtLeast(1), true) else raw
        } catch (e: Exception) {
            null
        }
    }

    /** Neuer Stand vom Player (aus dem Hintergrunddienst). */
    fun update(ctx: Context, args: MediaArgs, cover: Bitmap?) {
        if (args.radio) {
            ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
                .putLong("radioId", args.stationId)
                .putString("radioName", args.stationName)
                .putString("radioSong", args.title)
                .apply()
            val logoFile = File(ctx.cacheDir, "widget_radio.png")
            if (cover != null) logoFile.outputStream().use { cover.compress(Bitmap.CompressFormat.PNG, 100, it) } else logoFile.delete()
        }
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
            .putBoolean("radio", args.radio)
            .putString("title", args.title)
            .putString("artist", args.artist)
            .putString("album", args.album)
            .putBoolean("playing", args.playing)
            .putString("coverKey", args.title + "|" + args.artist + "|" + (args.cover?.length ?: 0))
            .apply()
        val file = File(ctx.cacheDir, COVER)
        if (cover != null) {
            try {
                // klein genug für Widgets (Android begrenzt die Bildgröße)
                val size = 320
                val scaled = if (cover.width > size || cover.height > size) Bitmap.createScaledBitmap(cover, size, size * cover.height / cover.width.coerceAtLeast(1), true) else cover
                file.outputStream().use { scaled.compress(Bitmap.CompressFormat.PNG, 100, it) }
            } catch (e: Exception) {
                file.delete()
            }
        } else {
            file.delete()
        }
        refresh(ctx)
    }

    /** Wiedergabe beendet (App geschlossen): Pause-Zustand zeigen. */
    fun markStopped(ctx: Context) {
        ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putBoolean("playing", false).apply()
        refresh(ctx)
    }

    fun refresh(ctx: Context) {
        val mgr = AppWidgetManager.getInstance(ctx)
        val prefs = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        val title = prefs.getString("title", null)?.takeIf { it.isNotEmpty() } ?: "Amber Player"
        val artist = prefs.getString("artist", "") ?: ""
        val album = prefs.getString("album", "") ?: ""
        val playing = prefs.getBoolean("playing", false) && AmberMobilePlugin.instance != null
        val coverFile = File(ctx.cacheDir, COVER)
        val cover = if (coverFile.exists()) BitmapFactory.decodeFile(coverFile.path) else null

        val layouts = listOf(
            AmberWidgetSmall::class.java to R.layout.amber_widget_small,
            AmberWidgetLarge::class.java to R.layout.amber_widget_large,
            AmberWidgetSquare::class.java to R.layout.amber_widget_square,
        )
        for ((cls, layout) in layouts) {
            val ids = mgr.getAppWidgetIds(ComponentName(ctx, cls))
            if (ids.isEmpty()) continue
            val v = RemoteViews(ctx.packageName, layout)
            v.setTextViewText(R.id.amber_title, title)
            v.setTextViewText(R.id.amber_artist, artist)
            if (layout == R.layout.amber_widget_large) v.setTextViewText(R.id.amber_album, album)
            if (cover != null) v.setImageViewBitmap(R.id.amber_cover, cover) else v.setImageViewResource(R.id.amber_cover, R.drawable.ic_amber_note)
            v.setImageViewResource(R.id.amber_play, if (playing) R.drawable.ic_amber_pause else R.drawable.ic_amber_play)
            v.setContentDescription(R.id.amber_play, ctx.getString(if (playing) R.string.amber_pause else R.string.amber_play))
            // Cover und Titel öffnen die App, die Tasten steuern die Wiedergabe
            open(ctx)?.let {
                v.setOnClickPendingIntent(R.id.amber_cover, it)
                v.setOnClickPendingIntent(R.id.amber_text, it)
            }
            v.setOnClickPendingIntent(R.id.amber_play, action(ctx, if (playing) "pause" else "play", 21))
            if (layout != R.layout.amber_widget_square) {
                v.setOnClickPendingIntent(R.id.amber_prev, action(ctx, "prev", 22))
                v.setOnClickPendingIntent(R.id.amber_next, action(ctx, "next", 23))
            }
            mgr.updateAppWidget(ids, v)
        }
        refreshRadio(ctx, mgr, prefs)
    }

    /** Radio-Widget: laufender bzw. zuletzt gehörter Sender, darunter die gespeicherten Sender. */
    private fun refreshRadio(ctx: Context, mgr: AppWidgetManager, prefs: android.content.SharedPreferences) {
        val ids = mgr.getAppWidgetIds(ComponentName(ctx, AmberWidgetRadio::class.java))
        if (ids.isEmpty()) return
        val alive = AmberMobilePlugin.instance != null
        val radioNow = prefs.getBoolean("radio", false) && alive
        val playing = radioNow && prefs.getBoolean("playing", false)
        val stationIds = prefs.getString("stationIds", "")!!.split(",").mapNotNull { it.toLongOrNull() }
        val stationNames = prefs.getString("stationNames", "")!!.split("\u001F")
        val lastId = prefs.getLong("radioId", stationIds.firstOrNull() ?: 0)
        val name = prefs.getString("radioName", null)?.takeIf { it.isNotEmpty() }
            ?: stationNames.getOrNull(stationIds.indexOf(lastId))?.takeIf { it.isNotEmpty() }
            ?: ctx.getString(R.string.amber_widget_radio_empty)
        val song = if (radioNow) prefs.getString("radioSong", "") ?: "" else ctx.getString(R.string.amber_widget_radio_hint)
        val v = RemoteViews(ctx.packageName, R.layout.amber_widget_radio)
        v.setTextViewText(R.id.amber_title, name)
        v.setTextViewText(R.id.amber_artist, if (song == name) "" else song)
        val logo = File(ctx.cacheDir, "widget_radio.png").takeIf { it.exists() }?.let { BitmapFactory.decodeFile(it.path) }
            ?: File(ctx.cacheDir, "widget_station_$lastId.png").takeIf { it.exists() }?.let { BitmapFactory.decodeFile(it.path) }
        if (logo != null) v.setImageViewBitmap(R.id.amber_cover, logo) else v.setImageViewResource(R.id.amber_cover, R.drawable.ic_amber_radio)
        v.setImageViewResource(R.id.amber_play, if (playing) R.drawable.ic_amber_pause else R.drawable.ic_amber_play)
        open(ctx)?.let {
            v.setOnClickPendingIntent(R.id.amber_cover, it)
            v.setOnClickPendingIntent(R.id.amber_text, it)
        }
        // Play: läuft Radio → Pause/Weiter; sonst den zuletzt gehörten Sender starten
        val playAction = when {
            playing -> "pause"
            radioNow -> "play"
            lastId > 0 -> "station:$lastId"
            else -> "play"
        }
        v.setOnClickPendingIntent(R.id.amber_play, action(ctx, playAction, 31))
        v.setOnClickPendingIntent(R.id.amber_prev, action(ctx, if (radioNow) "prev" else "station:$lastId", 32))
        v.setOnClickPendingIntent(R.id.amber_next, action(ctx, if (radioNow) "next" else "station:$lastId", 33))
        val slots = listOf(R.id.amber_st1, R.id.amber_st2, R.id.amber_st3, R.id.amber_st4, R.id.amber_st5)
        val labels = listOf(R.id.amber_st1_name, R.id.amber_st2_name, R.id.amber_st3_name, R.id.amber_st4_name, R.id.amber_st5_name)
        val boxes = listOf(R.id.amber_st1_box, R.id.amber_st2_box, R.id.amber_st3_box, R.id.amber_st4_box, R.id.amber_st5_box)
        for (i in slots.indices) {
            val id = stationIds.getOrNull(i)
            if (id == null) {
                v.setViewVisibility(boxes[i], android.view.View.INVISIBLE)
                continue
            }
            v.setViewVisibility(boxes[i], android.view.View.VISIBLE)
            val f = File(ctx.cacheDir, "widget_station_$id.png")
            if (f.exists()) v.setImageViewBitmap(slots[i], BitmapFactory.decodeFile(f.path)) else v.setImageViewResource(slots[i], R.drawable.ic_amber_radio)
            v.setTextViewText(labels[i], stationNames.getOrNull(i) ?: "")
            v.setOnClickPendingIntent(boxes[i], action(ctx, "station:$id", 40 + i))
        }
        v.setViewVisibility(R.id.amber_stations_empty, if (stationIds.isEmpty()) android.view.View.VISIBLE else android.view.View.GONE)
        mgr.updateAppWidget(ids, v)
    }

    private fun open(ctx: Context): PendingIntent? =
        ctx.packageManager.getLaunchIntentForPackage(ctx.packageName)?.let {
            PendingIntent.getActivity(ctx, 20, it, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        }

    private fun action(ctx: Context, name: String, code: Int): PendingIntent {
        // Läuft der Player nicht, öffnen auch die Tasten die App (ein Tipp genügt).
        if (AmberMobilePlugin.instance == null) {
            open(ctx)?.let { return it }
        }
        val intent = Intent(ctx, AmberWidgetAction::class.java).putExtra("action", name)
        return PendingIntent.getBroadcast(ctx, code, intent, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
    }
}
