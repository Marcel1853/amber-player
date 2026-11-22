package de.amberplayer.mobile

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.ServiceInfo
import android.media.audiofx.AudioEffect
import android.media.audiofx.DynamicsProcessing
import android.media.audiofx.EnvironmentalReverb
import android.media.audiofx.LoudnessEnhancer
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat

/**
 * Equalizer von Amber Player für andere Apps (wie ein System-Equalizer).
 *
 * Musik-Apps melden ihre Audio-Sitzung per Rundruf („Equalizer-Sitzung geöffnet“); an diese
 * Sitzungen hängt Amber Player einen Android-Audioeffekt (DynamicsProcessing) mit den Werten des
 * eigenen 10-Band-Equalizers und der Lautstärke-Anhebung – samt Begrenzer gegen Übersteuern.
 * Wahlweise wirkt er global auf den gesamten Ton-Mix (Sitzung 0; geht nicht auf jedem Gerät).
 *
 * Solange er an ist, zeigt eine Benachrichtigung das an; „Ausschalten“ beendet ihn. Wieder an
 * geht nur in der App.
 */
object GlobalEq {
    private const val PREFS = "amber_global_eq"
    /** Bandzahlen, die nacheinander probiert werden (manche Geräte nehmen nicht beliebig viele) */
    private val BAND_COUNTS = intArrayOf(31, 16, 10)

    /** Klangkurve, Lautstärke-Verstärker und Hall einer Sitzung */
    private class Fx(val dp: DynamicsProcessing, val bands: Int, var loud: LoudnessEnhancer?, var reverb: EnvironmentalReverb?) {
        /** Eingangspegel links/rechts ohne Auto-Pan (Balance minus Spielraum) */
        val base = FloatArray(2)
        /** Lautstärke-Verstärker ließ sich (noch) nicht anlegen – später erneut versuchen */
        var loudPending = false
    }

    /** Was Amber gerade an andere Apps gibt (vom Player gemessen) */
    private class Profile(
        val centers: FloatArray, val gains: FloatArray, val balance: FloatArray, val boost: Float,
        val reverb: org.json.JSONObject?, val ceiling: Float,
        /** Auto-Pan (8D-Rundflug): je Eintrag Tempo (Hz) und Tiefe (0…1) */
        val autopan: List<Pair<Float, Float>>,
    )

    @Volatile private var cached: Profile? = null

    /** Sitzung → Effekte (nur solange der Dienst läuft) */
    private val effects = mutableMapOf<Int, Fx>()
    /** Sitzung → App (für die Anzeige) */
    private val owners = mutableMapOf<Int, String>()

    fun supported(): Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.P

    /**
     * Der Dienst läuft in einem eigenen Prozess (`:effects`), damit er weiterläuft, wenn die App
     * aus der Übersicht gewischt wird. Darum liegt alles Gemeinsame in kleinen JSON-Dateien –
     * SharedPreferences sind zwischen Prozessen nicht verlässlich.
     */
    private fun file(ctx: Context, name: String) = java.io.File(ctx.filesDir, "effects-$name.json")

    private fun readJson(ctx: Context, name: String): org.json.JSONObject = try {
        org.json.JSONObject(file(ctx, name).readText())
    } catch (e: Exception) {
        org.json.JSONObject()
    }

    private fun writeJson(ctx: Context, name: String, text: String) {
        try {
            val f = file(ctx, name)
            val tmp = java.io.File(f.path + ".tmp")
            tmp.writeText(text)
            tmp.renameTo(f)
        } catch (e: Exception) {
            android.util.Log.w("AmberPlayer", "Effekte für andere Apps: $name speichern – ${e.message}")
        }
    }

    private fun state(ctx: Context): org.json.JSONObject {
        if (!file(ctx, "state").exists()) {
            // frühere Version: Einstellung aus SharedPreferences übernehmen
            val old = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
            if (old.contains("enabled")) {
                return org.json.JSONObject().put("enabled", old.getBoolean("enabled", false)).put("mode", old.getString("mode", "apps"))
            }
        }
        return readJson(ctx, "state")
    }

    fun enabled(ctx: Context) = state(ctx).optBoolean("enabled", false)
    fun mode(ctx: Context) = state(ctx).optString("mode", "apps").ifEmpty { "apps" }

    /** Ausgeschaltet über die Benachrichtigung (Dienst-Prozess). */
    fun setOff(ctx: Context) {
        writeJson(ctx, "state", state(ctx).put("enabled", false).toString())
    }

    /** Meldung ins Fehlerprotokoll – aus dem Dienst-Prozess per Rundruf an die App. */
    fun log(message: String) {
        val plugin = AmberMobilePlugin.instance
        if (plugin != null) {
            plugin.note(message)
            return
        }
        android.util.Log.w("AmberPlayer", message)
        appCtx?.let { ctx ->
            try {
                ctx.sendBroadcast(Intent(NOTE_ACTION).setPackage(ctx.packageName).putExtra("message", message))
            } catch (_: Exception) {
            }
        }
    }
    const val NOTE_ACTION = "de.amberplayer.app.EFFECTS_NOTE"
    @Volatile var appCtx: Context? = null

    /** Für die Einstellungen: Apps, an denen der Dienst gerade hängt (vom Dienst geschrieben). */
    fun statusApps(ctx: Context): List<String> {
        val a = readJson(ctx, "status").optJSONArray("apps") ?: return emptyList()
        return (0 until a.length()).map { a.optString(it) }
    }

    fun writeStatus(ctx: Context, labels: List<String>) {
        writeJson(ctx, "status", org.json.JSONObject().put("apps", org.json.JSONArray(labels)).toString())
    }

    /** Ein- oder ausschalten (nur aus der App) und die Art wählen: „apps“ oder „global“. */
    fun setEnabled(ctx: Context, on: Boolean, mode: String) {
        writeJson(ctx, "state", org.json.JSONObject()
            .put("enabled", on && supported())
            .put("mode", if (mode == "global") "global" else "apps").toString())
        if (on && supported()) {
            ContextCompat.startForegroundService(ctx, Intent(ctx, GlobalEqService::class.java).setAction("apply"))
        } else {
            ctx.stopService(Intent(ctx, GlobalEqService::class.java))
        }
    }

    /** Messung vom Player: Klangkurve (Terzbänder), Balance, Hall und Anhebung als JSON. */
    @Synchronized
    fun setProfile(ctx: Context, json: String) {
        writeJson(ctx, "profile", json)
        cached = null
        if (enabled(ctx)) effects.forEach { (session, fx) -> configureEffect(ctx, session, fx) }
    }

    private fun profile(ctx: Context): Profile {
        val stamp = file(ctx, "profile").lastModified()
        if (stamp != cachedStamp) {
            cached = null
            cachedStamp = stamp
        }
        return cached ?: parseProfile(ctx).also { cached = it }
    }
    @Volatile private var cachedStamp = -1L

    /** Hat die App eine neue Messung geschrieben? (Dienst-Prozess, vom Takt aus geprüft) */
    fun profileChanged(ctx: Context): Boolean = file(ctx, "profile").lastModified() != cachedStamp

    private fun parseProfile(ctx: Context): Profile {
        fun arr(a: org.json.JSONArray?): FloatArray = FloatArray(a?.length() ?: 0) { a!!.optDouble(it, 0.0).toFloat() }
        return try {
            val o = readJson(ctx, "profile")
            Profile(arr(o.optJSONArray("centers")), arr(o.optJSONArray("gains")), arr(o.optJSONArray("balance")),
                o.optDouble("boost", 0.0).toFloat(), o.optJSONObject("reverb"),
                if (o.isNull("ceiling")) -0.5f else o.optDouble("ceiling", -0.5).toFloat(),
                (o.optJSONArray("autopan") ?: org.json.JSONArray()).let { a ->
                    (0 until a.length()).mapNotNull { i ->
                        a.optJSONArray(i)?.let { it.optDouble(0, 0.2).toFloat() to it.optDouble(1, 0.0).toFloat() }
                    }
                })
        } catch (e: Exception) {
            Profile(FloatArray(0), FloatArray(0), FloatArray(0), 0f, null, -0.5f, emptyList())
        }
    }

    /** Verstärkung der Kurve bei einer Frequenz (linear über log-Frequenz zwischen den Messpunkten). */
    private fun gainAt(p: Profile, f: Float): Float {
        val c = p.centers
        if (c.isEmpty() || p.gains.size != c.size) return 0f
        if (f <= c[0]) return p.gains[0]
        if (f >= c[c.size - 1]) return p.gains[c.size - 1]
        val i = c.indexOfFirst { it >= f }
        val t = (Math.log((f / c[i - 1]).toDouble()) / Math.log((c[i] / c[i - 1]).toDouble())).toFloat()
        return p.gains[i - 1] + t * (p.gains[i] - p.gains[i - 1])
    }

    /**
     * Erweiterte Erkennung: Mit der Sonderfreigabe „DUMP“ (einmalig per adb erteilt) liest Amber
     * Player die Liste aller laufenden Wiedergaben von Android – so werden auch Apps erkannt, die
     * sich nicht beim Equalizer melden (z. B. YouTube).
     */
    fun dumpGranted(ctx: Context) =
        ctx.checkSelfPermission("android.permission.DUMP") == android.content.pm.PackageManager.PERMISSION_GRANTED

    /** Über die erweiterte Erkennung gefundene Sitzungen */
    private val scanned = mutableSetOf<Int>()
    private val PKG = Regex("""uid/pid:(\d+)/\d+ package:([\w.]+)""")
    private val PLAYER = Regex("""AudioPlaybackConfiguration piid:\d+ .*?u/pid:(\d+)/\d+ state:(\w+) attr:.*?usage=(\w+).*?sessionId:(\d+)""")

    /** Laufende Wiedergaben anderer Apps suchen und den Equalizer anhängen. */
    fun scan(ctx: Context) {
        if (!enabled(ctx) || mode(ctx) != "apps" || !dumpGranted(ctx)) return
        val text = try {
            val proc = Runtime.getRuntime().exec(arrayOf("dumpsys", "audio"))
            val out = proc.inputStream.bufferedReader().readText()
            proc.waitFor()
            out
        } catch (e: Exception) {
            return
        }
        val section = text.substringAfter("  players:", "").substringBefore("ducked players", "")
        // Paketnamen aus dem Ereignisprotokoll (falls Android uns die App sonst nicht zeigt)
        val pkgOfUid = PKG.findAll(text).associate { it.groupValues[1].toInt() to it.groupValues[2] }
        val seen = mutableSetOf<Int>()
        val myUid = android.os.Process.myUid()
        for (m in PLAYER.findAll(section)) {
            val (uidS, state, usage, sessionS) = m.destructured
            val uid = uidS.toIntOrNull() ?: continue
            val session = sessionS.toIntOrNull() ?: continue
            if (session <= 0 || uid == myUid || uid < 10000) continue
            if (usage != "USAGE_MEDIA" && usage != "USAGE_GAME" && usage != "USAGE_UNKNOWN") continue
            seen += session
            if (state != "started") continue
            val known = synchronized(this) { effects.containsKey(session) }
            if (!known) {
                val pkg = ctx.packageManager.getPackagesForUid(uid)?.firstOrNull() ?: pkgOfUid[uid] ?: "uid $uid"
                attach(ctx, session, pkg)
                if (synchronized(this) { effects.containsKey(session) }) synchronized(this) { scanned += session }
            }
        }
        // beendete Wiedergaben freigeben
        val gone = synchronized(this) { scanned.filter { it !in seen }.also { scanned.removeAll(it.toSet()) } }
        if (gone.isNotEmpty()) {
            gone.forEach { release(it) }
            GlobalEqService.refresh(ctx)
        }
    }

    @Synchronized
    fun apps(): List<String> = owners.values.distinct().sorted()

    /** Alle Effekte neu einstellen bzw. für die gewählte Art anlegen. */
    @Synchronized
    fun applyAll(ctx: Context) {
        if (!supported()) return
        if (mode(ctx) == "global") {
            // nur Sitzung 0, Einzel-Sitzungen freigeben
            effects.keys.filter { it != 0 }.forEach { release(it) }
            attach(ctx, 0, "Global")
        } else {
            release(0)
        }
        effects.forEach { (session, fx) -> configureEffect(ctx, session, fx) }
    }

    @Synchronized
    fun attach(ctx: Context, session: Int, pkg: String) {
        if (!supported() || !enabled(ctx)) return
        if (pkg == ctx.packageName) return // Amber Player hat seinen eigenen Equalizer
        if (mode(ctx) == "global" && session != 0) return
        if (effects.containsKey(session)) {
            owners[session] = pkg
            return
        }
        // Neue Sitzung derselben App: die alte ist veraltet (sonst hängt Amber an der falschen)
        if (pkg != "?" && session != 0) {
            owners.filter { (s, p) -> p == pkg && s != session && s != 0 }.keys.toList().forEach { old ->
                release(old)
                forget(ctx, old)
                log("Effekte für andere Apps: alte Sitzung $old von „$pkg“ freigegeben")
            }
        }
        owners[session] = pkg
        try {
            // Ab Android 15 (neues AIDL-Audiosystem, z. B. Samsung) läuft
            // DynamicsProcessing nur mit der Zeitauflösungs-Variante zuverlässig, und sein Begrenzer
            // ist dort fehlerhaft. Darum ohne Begrenzer; die Anhebung übernimmt LoudnessEnhancer,
            // der selbst gegen Übersteuern schützt.
            val preferTime = Build.VERSION.SDK_INT >= 35
            val variants = if (preferTime) listOf(DynamicsProcessing.VARIANT_FAVOR_TIME_RESOLUTION, DynamicsProcessing.VARIANT_FAVOR_FREQUENCY_RESOLUTION)
            else listOf(DynamicsProcessing.VARIANT_FAVOR_FREQUENCY_RESOLUTION, DynamicsProcessing.VARIANT_FAVOR_TIME_RESOLUTION)
            var made: Pair<DynamicsProcessing, Int>? = null
            var lastError: Exception? = null
            loop@ for (n in BAND_COUNTS) for (v in variants) {
                try {
                    val cfg = DynamicsProcessing.Config.Builder(v, 2, true, n, false, 0, false, 0, false).build()
                    made = DynamicsProcessing(Int.MAX_VALUE, session, cfg) to n
                    log(
                        "Effekte für andere Apps: $n Bänder, ${if (v == DynamicsProcessing.VARIANT_FAVOR_TIME_RESOLUTION) "Zeitauflösung" else "Frequenzauflösung"} (Android ${Build.VERSION.SDK_INT})"
                    )
                    break@loop
                } catch (e: Exception) {
                    lastError = e
                }
            }
            val (dp, bands) = made ?: throw (lastError ?: IllegalStateException("kein Equalizer möglich"))
            val fx = Fx(dp, bands, null, null)
            configureEffect(ctx, session, fx)
            dp.enabled = true
            effects[session] = fx
            remember(ctx, session, pkg)
            log("Equalizer für andere Apps: an „$pkg“ (Sitzung $session) angehängt")
        } catch (e: Exception) {
            owners.remove(session)
            log("Equalizer für andere Apps: „$pkg“ ging nicht – ${e.message}")
        }
        GlobalEqService.refresh(ctx)
    }

    @Synchronized
    fun release(session: Int) {
        effects.remove(session)?.let { fx ->
            for (e in listOfNotNull<AudioEffect>(fx.dp, fx.loud, fx.reverb)) {
                try {
                    e.enabled = false
                    e.release()
                } catch (_: Exception) {
                }
            }
        }
        owners.remove(session)
    }

    /** Gemeldete Sitzungen merken, damit Amber nach einem Neustart/Update wieder andockt. */
    private fun remember(ctx: Context, session: Int, pkg: String) {
        if (session == 0) return
        val o = readJson(ctx, "sessions")
        o.put(session.toString(), pkg)
        writeJson(ctx, "sessions", o.toString())
    }

    /** Sitzung wurde von der App geschlossen: nicht mehr merken. */
    fun forget(ctx: Context, session: Int) {
        val o = readJson(ctx, "sessions")
        o.remove(session.toString())
        writeJson(ctx, "sessions", o.toString())
    }

    /**
     * „Apps neu suchen“: alles lösen, je App nur die neueste gemerkte Sitzung behalten und neu
     * anhängen (holt dabei auch die Kontrolle zurück, falls eine andere Equalizer-App sie hat).
     * Mit Sonderfreigabe werden zusätzlich alle laufenden Wiedergaben durchsucht.
     */
    fun rescan(ctx: Context) {
        releaseAll()
        if (mode(ctx) == "global") {
            applyAll(ctx)
            return
        }
        val o = readJson(ctx, "sessions")
        val newest = linkedMapOf<String, String>() // App → Sitzung (die zuletzt gemerkte gewinnt)
        for (key in o.keys()) newest[o.optString(key, "?")] = key
        val kept = org.json.JSONObject()
        newest.forEach { (pkg, session) -> kept.put(session, pkg) }
        writeJson(ctx, "sessions", kept.toString())
        synchronized(this) { scanned.clear() }
        reattach(ctx)
        scan(ctx)
        log("Effekte für andere Apps: neu gesucht – ${apps().ifEmpty { listOf("keine App") }.joinToString(", ")}")
    }

    /** Gemerkte Sitzungen wieder anhängen (beim Start des Dienstes). */
    fun reattach(ctx: Context) {
        if (!enabled(ctx) || mode(ctx) != "apps") return
        val o = if (file(ctx, "sessions").exists()) readJson(ctx, "sessions") else try {
            org.json.JSONObject(ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString("sessions", "{}") ?: "{}")
        } catch (e: Exception) {
            org.json.JSONObject()
        }
        for (key in o.keys()) {
            val session = key.toIntOrNull() ?: continue
            attach(ctx, session, o.optString(key, "?"))
        }
    }

    /**
     * Ohne Sonderfreigabe: Androids öffentliche Liste der laufenden Wiedergaben. Je nach Gerät
     * steht darin die Sitzungsnummer; dann hängt Amber sich auch an Apps, die schon liefen.
     */
    fun scanPublic(ctx: Context) {
        if (publicUseless || !enabled(ctx) || mode(ctx) != "apps" || dumpGranted(ctx)) return
        val am = ctx.getSystemService(android.media.AudioManager::class.java) ?: return
        val configs = try { am.activePlaybackConfigurations } catch (e: Exception) { return }
        for (c in configs) {
            val text = c.toString()
            // Anonymisiert (wie auf den meisten Geräten)? Dann bringt diese Liste nichts – nicht
            // mehr abfragen, das spart Akku.
            if (text.contains("state:started") && text.contains("u/pid:-1/-1") && text.contains("sessionId:0")) {
                publicUseless = true
                log("Effekte für andere Apps: öffentliche Wiedergabeliste ist auf diesem Gerät anonymisiert")
                return
            }
            val usage = c.audioAttributes.usage
            if (usage != android.media.AudioAttributes.USAGE_MEDIA && usage != android.media.AudioAttributes.USAGE_GAME) continue
            val session = Regex("""sessionId:(\d+)""").find(text)?.groupValues?.get(1)?.toIntOrNull() ?: continue
            if (session <= 0) continue
            if (synchronized(this) { effects.containsKey(session) }) continue
            attach(ctx, session, "?")
        }
    }
    @Volatile private var publicUseless = false

    /** Lohnt sich das regelmäßige Suchen? (Sonderfreigabe oder brauchbare öffentliche Liste) */
    fun scanUseful(ctx: Context) = enabled(ctx) && mode(ctx) == "apps" && (dumpGranted(ctx) || !publicUseless)

    /** Muss der Takt schnell laufen? Nur für das Kreisen vom 8D-Rundflug. */
    @Synchronized
    fun panning(ctx: Context) = effects.isNotEmpty() && profile(ctx).autopan.isNotEmpty()

    /**
     * Auto-Pan (8D-Rundflug) für andere Apps: die Balance wandert im Tempo des Effekts – wie in
     * Amber (links leiser, wenn es nach rechts geht und umgekehrt). Läuft etwa 30-mal pro Sekunde.
     */
    @Synchronized
    fun tick(ctx: Context, seconds: Double) {
        if (effects.isEmpty()) return
        if (seconds - lastProfileCheck >= 0.5) {
            lastProfileCheck = seconds
            if (profileChanged(ctx)) effects.forEach { (session, fx) -> configureEffect(ctx, session, fx) }
        }
        if (seconds - lastRetry >= 2.0) {
            lastRetry = seconds
            effects.forEach { (session, fx) ->
                if (fx.loudPending) {
                    configureEffect(ctx, session, fx, quiet = true)
                    if (!fx.loudPending) log("Effekte für andere Apps: Begrenzer für Sitzung $session jetzt aktiv")
                }
            }
        }
        val pans = profile(ctx).autopan
        if (pans.isEmpty()) {
            if (panActive) {
                panActive = false
                effects.values.forEach { fx -> for (ch in 0..1) try { fx.dp.setInputGainbyChannel(ch, fx.base[ch]) } catch (_: Exception) {} }
            }
            return
        }
        panActive = true
        var gl = 1.0
        var gr = 1.0
        for ((rate, depth) in pans) {
            val pan = Math.sin(2 * Math.PI * ((seconds * rate) % 1.0)) * depth
            gl *= minOf(1.0, 1.0 - pan)
            gr *= minOf(1.0, 1.0 + pan)
        }
        val dl = (20 * Math.log10(maxOf(gl, 0.001))).toFloat()
        val dr = (20 * Math.log10(maxOf(gr, 0.001))).toFloat()
        for (fx in effects.values) {
            try {
                fx.dp.setInputGainbyChannel(0, fx.base[0] + dl)
                fx.dp.setInputGainbyChannel(1, fx.base[1] + dr)
            } catch (_: Exception) {
            }
        }
    }
    private var panActive = false
    private var lastRetry = 0.0
    private var lastProfileCheck = 0.0

    @Synchronized
    fun releaseAll() {
        effects.keys.toList().forEach { release(it) }
    }

    /** Klangkurve, Balance, Anhebung und Hall auf die Effekte einer Sitzung übertragen. */
    private fun configureEffect(ctx: Context, session: Int, fx: Fx, quiet: Boolean = false) {
        val p = profile(ctx)
        // Bänder gleichmäßig (logarithmisch) von 20 Hz bis 20 kHz
        val lo = 20.0
        val hi = 20000.0
        val ratio = Math.pow(hi / lo, 1.0 / fx.bands)
        // Gegen Übersteuern: die Kurve um ihre stärkste Anhebung absenken (Spielraum) und die
        // Lautstärke danach mit LoudnessEnhancer zurückholen – der begrenzt selbst und ersetzt
        // so den „Vorverstärker & Limiter“ (der Begrenzer von Dynamics Processing ist auf
        // Samsung/Android 15+ fehlerhaft). Die Obergrenze senkt das Ziel entsprechend.
        var headroom = 0f
        try {
            val gains = FloatArray(fx.bands) { i -> gainAt(p, (lo * Math.pow(ratio, i + 0.5)).toFloat()) }
            headroom = maxOf(0f, gains.maxOrNull() ?: 0f) + maxOf(0f, p.balance.maxOrNull() ?: 0f)
            for (i in 0 until fx.bands) {
                val upper = (lo * Math.pow(ratio, (i + 1).toDouble())).toFloat()
                fx.dp.setPreEqBandAllChannelsTo(i, DynamicsProcessing.EqBand(true, upper, gains[i]))
            }
            for (ch in 0..1) {
                fx.base[ch] = p.balance.getOrElse(ch) { 0f } - headroom
                fx.dp.setInputGainbyChannel(ch, fx.base[ch])
            }
        } catch (e: Exception) {
            log("Effekte für andere Apps: Klangkurve ging nicht – ${e.message}")
        }
        val target = headroom + p.boost + p.ceiling.coerceIn(-6f, 0f)
        fx.loudPending = false
        try {
            if (target > 0.05f) {
                val loud = fx.loud ?: LoudnessEnhancer(session).also { fx.loud = it }
                loud.setTargetGain((target * 100).toInt())
                loud.enabled = true
            } else {
                fx.loud?.enabled = false
            }
        } catch (e: Exception) {
            // Geht z. B. nicht, solange die App pausiert. Dann ohne Spielraum (nicht leiser) und
            // gleich nochmal versuchen.
            fx.loudPending = true
            if (headroom > 0f) {
                for (ch in 0..1) {
                    fx.base[ch] += headroom
                    try { fx.dp.setInputGainbyChannel(ch, fx.base[ch]) } catch (_: Exception) {}
                }
            }
        }
        configureReverb(session, fx, p.reverb)
        if (!quiet) log(
            "Effekte für andere Apps: Sitzung $session – Kurve ${p.gains.joinToString(" ") { "%.0f".format(it) }} dB, " +
                "Balance ${p.balance.joinToString("/") { "%.1f".format(it) }}, Spielraum %.1f dB, Verstärker %.1f dB, Hall ${if (p.reverb != null) "an" else "aus"}".format(headroom, target)
        )
    }

    /** Ambers Hall auf Androids eingebauten Hall abbilden (angenähert). */
    private fun configureReverb(session: Int, fx: Fx, r: org.json.JSONObject?) {
        if (r == null) {
            try { fx.reverb?.enabled = false } catch (_: Exception) {}
            return
        }
        val rv = try {
            fx.reverb ?: EnvironmentalReverb(Int.MAX_VALUE, session).also { fx.reverb = it }
        } catch (e: Exception) {
            log("Effekte für andere Apps: Hall nicht verfügbar – ${e.message}")
            return
        }
        val size = r.optDouble("size", 45.0)
        val damp = r.optDouble("damp", 60.0)
        val decay = r.optDouble("decay", 1.6)
        val predelay = r.optDouble("predelay", 20.0)
        val mix = r.optDouble("mix", 25.0).coerceIn(0.0, 100.0)
        fun set(what: String, block: () -> Unit) = try { block() } catch (e: Exception) {
            log("Effekte für andere Apps: Hall-$what ging nicht – ${e.message}")
        }
        val wetMb = if (mix < 0.5) -9000 else (2000 * Math.log10(mix / 100.0)).toInt().coerceIn(-9000, 0)
        set("Pegel") { rv.roomLevel = 0 }
        set("Höhen") { rv.roomHFLevel = (-damp * 20).toInt().coerceIn(-9000, 0).toShort() }
        set("Nachhall") { rv.decayTime = (decay * 1000).toInt().coerceIn(100, 7000) }
        set("Dämpfung") { rv.decayHFRatio = (1500 - damp * 12).toInt().coerceIn(100, 2000).toShort() }
        set("Reflexionen") { rv.reflectionsLevel = (wetMb - 600).coerceIn(-9000, 1000).toShort() }
        set("Reflexionszeit") { rv.reflectionsDelay = (size * 0.8).toInt().coerceIn(0, 300) }
        set("Hallpegel") { rv.reverbLevel = wetMb.toShort() }
        set("Verzögerung") { rv.reverbDelay = predelay.toInt().coerceIn(0, 100) }
        set("Diffusion") { rv.diffusion = 1000 }
        set("Dichte") { rv.density = (size * 10).toInt().coerceIn(0, 1000).toShort() }
        set("Einschalten") { rv.enabled = true }
    }
}

/**
 * Empfängt „Equalizer-Sitzung geöffnet/geschlossen“ von Musik-Apps. Wird vom Dienst angemeldet,
 * solange er läuft (im Manifest eingetragene Empfänger bekommen solche Rundrufe seit Android 8
 * nicht mehr).
 */
class AudioSessionReceiver : BroadcastReceiver() {
    override fun onReceive(ctx: Context, intent: Intent) {
        if (!GlobalEq.enabled(ctx)) return
        val session = intent.getIntExtra(AudioEffect.EXTRA_AUDIO_SESSION, AudioEffect.ERROR_BAD_VALUE)
        val pkg = intent.getStringExtra(AudioEffect.EXTRA_PACKAGE_NAME) ?: "?"
        if (session < 0) return
        when (intent.action) {
            AudioEffect.ACTION_OPEN_AUDIO_EFFECT_CONTROL_SESSION -> GlobalEq.attach(ctx, session, pkg)
            AudioEffect.ACTION_CLOSE_AUDIO_EFFECT_CONTROL_SESSION -> {
                GlobalEq.release(session)
                GlobalEq.forget(ctx, session)
                GlobalEqService.refresh(ctx)
            }
        }
    }
}

/** Hält den Equalizer am Leben und zeigt die Benachrichtigung mit „Ausschalten“. */
class GlobalEqService : Service() {
    companion object {
        private const val CHANNEL = "amber_global_eq"
        private const val ID = 7879
        private var running = false
        private val receiver = AudioSessionReceiver()
        @Volatile private var instance: GlobalEqService? = null

        fun refresh(ctx: Context) {
            if (running) ctx.startService(Intent(ctx, GlobalEqService::class.java).setAction("refresh"))
        }
    }

    override fun onCreate() {
        super.onCreate()
        running = true
        instance = this
        GlobalEq.appCtx = applicationContext
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val ch = NotificationChannel(CHANNEL, getString(R.string.amber_geq_channel), NotificationManager.IMPORTANCE_LOW)
            ch.setShowBadge(false)
            getSystemService(NotificationManager::class.java).createNotificationChannel(ch)
        }
        val filter = IntentFilter().apply {
            addAction(AudioEffect.ACTION_OPEN_AUDIO_EFFECT_CONTROL_SESSION)
            addAction(AudioEffect.ACTION_CLOSE_AUDIO_EFFECT_CONTROL_SESSION)
        }
        ContextCompat.registerReceiver(this, receiver, filter, ContextCompat.RECEIVER_EXPORTED)
        // Erkennung: mit Sonderfreigabe alle 4 Sekunden nachsehen, welche Apps spielen. Ohne
        // brauchbare Quelle nur selten prüfen, ob die Freigabe inzwischen erteilt wurde (Akku).
        val ctx = applicationContext
        scanner = Thread {
            while (!Thread.currentThread().isInterrupted) {
                try {
                    GlobalEq.scan(ctx)
                    GlobalEq.scanPublic(ctx)
                    Thread.sleep(if (GlobalEq.scanUseful(ctx)) 4000 else 30000)
                } catch (e: InterruptedException) {
                    break
                } catch (e: Exception) {
                }
            }
        }.apply {
            name = "eq-erkennung"
            isDaemon = true
            start()
        }
        // Auto-Pan (8D-Rundflug) für andere Apps
        val start = System.nanoTime()
        panner = Thread {
            while (!Thread.currentThread().isInterrupted) {
                try {
                    val t = (System.nanoTime() - start) / 1e9
                    GlobalEq.tick(ctx, t)
                    // weggewischte Benachrichtigung zurückholen, sobald wieder Musik läuft
                    if (hidden && t - lastMusicCheck >= 1.0) {
                        lastMusicCheck = t
                        if (musicPlaying()) {
                            hidden = false
                            show()
                        }
                    }
                    // schnell nur fürs Kreisen vom 8D-Rundflug, sonst selten (spart Akku)
                    Thread.sleep(if (GlobalEq.panning(ctx)) 33 else 500)
                } catch (e: InterruptedException) {
                    break
                } catch (e: Exception) {
                }
            }
        }.apply {
            name = "eq-rundflug"
            isDaemon = true
            start()
        }
    }

    private var panner: Thread? = null
    /** Benachrichtigung wurde weggewischt, während keine Musik lief */
    @Volatile private var hidden = false
    private var lastMusicCheck = 0.0

    /** Läuft gerade irgendwo Musik? (öffentliche Liste der Wiedergaben) */
    private fun musicPlaying(): Boolean = try {
        getSystemService(android.media.AudioManager::class.java).activePlaybackConfigurations.any {
            it.audioAttributes.usage == android.media.AudioAttributes.USAGE_MEDIA && it.toString().contains("state:started")
        }
    } catch (e: Exception) {
        false
    }

    private var scanner: Thread? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == "dismissed") {
            // Weggewischt: Solange Musik läuft, sofort wieder zeigen (sonst wüsste man nicht mehr,
            // dass die Effekte aktiv sind und wie man sie ausschaltet). Ohne Musik darf sie weg
            // bleiben – sie kommt zurück, sobald wieder etwas spielt.
            if (musicPlaying()) show() else hidden = true
            return START_STICKY
        }
        if (intent?.action == "rescan") {
            if (GlobalEq.enabled(this)) GlobalEq.rescan(this)
            show()
            return START_STICKY
        }
        if (intent?.action == "off") {
            // Ausschalten aus der Benachrichtigung: Effekte aus, bis man sie in der App wieder anmacht
            GlobalEq.setOff(this)
            GlobalEq.log("Effekte für andere Apps über die Benachrichtigung ausgeschaltet")
            show() // startForeground muss auch hier einmal passieren
            stopSelf()
            return START_NOT_STICKY
        }
        if (!GlobalEq.enabled(this)) {
            show()
            stopSelf()
            return START_NOT_STICKY
        }
        GlobalEq.applyAll(this)
        GlobalEq.reattach(this)
        show()
        return START_STICKY
    }

    private fun show() {
        val apps = GlobalEq.apps()
        GlobalEq.writeStatus(this, apps.map { label(it) })
        val text = when {
            GlobalEq.mode(this) == "global" -> getString(R.string.amber_geq_global)
            apps.isEmpty() -> getString(R.string.amber_geq_waiting)
            else -> getString(R.string.amber_geq_apps, apps.joinToString(", ") { label(it) })
        }
        val off = PendingIntent.getService(
            this, 1, Intent(this, GlobalEqService::class.java).setAction("off"),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val open = packageManager.getLaunchIntentForPackage(packageName)?.let {
            PendingIntent.getActivity(this, 2, it, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        }
        val n = NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_amber_eq)
            .setContentTitle(getString(R.string.amber_geq_title))
            .setContentText(text)
            .setContentIntent(open)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .addAction(
                0, getString(R.string.amber_geq_rescan),
                PendingIntent.getService(
                    this, 4, Intent(this, GlobalEqService::class.java).setAction("rescan"),
                    PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
                ),
            )
            .addAction(0, getString(R.string.amber_geq_off), off)
            .setDeleteIntent(
                PendingIntent.getService(
                    this, 3, Intent(this, GlobalEqService::class.java).setAction("dismissed"),
                    PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
                )
            )
            .build()
        val type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK else 0
        ServiceCompat.startForeground(this, ID, n, type)
    }

    /** Anzeigename einer App (sonst der Paketname). */
    private fun label(pkg: String): String = try {
        packageManager.getApplicationLabel(packageManager.getApplicationInfo(pkg, 0)).toString()
    } catch (e: Exception) {
        pkg
    }

    override fun onDestroy() {
        instance = null
        GlobalEq.writeStatus(this, emptyList())
        scanner?.interrupt()
        panner?.interrupt()
        try {
            unregisterReceiver(receiver)
        } catch (_: Exception) {
        }
        GlobalEq.releaseAll()
        running = false
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
