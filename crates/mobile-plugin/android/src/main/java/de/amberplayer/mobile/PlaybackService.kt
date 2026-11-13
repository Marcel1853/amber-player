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
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.AudioManager
import android.os.Build
import android.os.IBinder
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.util.Base64
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import androidx.media.AudioAttributesCompat
import androidx.media.AudioFocusRequestCompat
import androidx.media.AudioManagerCompat
import androidx.media.session.MediaButtonReceiver

/**
 * Hält Amber Player im Hintergrund am Leben, solange Musik läuft, und zeigt die
 * Medien-Benachrichtigung (auch auf dem Sperrbildschirm). Tasten der Benachrichtigung,
 * Kopfhörer und Bluetooth gehen über die MediaSession an die Oberfläche.
 */
class PlaybackService : Service() {

    companion object {
        private const val CHANNEL = "amber_playback"
        private const val NOTIFICATION_ID = 7878

        private var last: MediaArgs? = null
        private var running = false

        fun update(ctx: Context, args: MediaArgs) {
            last = args
            val intent = Intent(ctx, PlaybackService::class.java).setAction("update")
            if (running) ctx.startService(intent) else ContextCompat.startForegroundService(ctx, intent)
        }

        fun stop(ctx: Context) {
            last = null
            ctx.stopService(Intent(ctx, PlaybackService::class.java))
        }
    }

    private lateinit var session: MediaSessionCompat
    private lateinit var audio: AudioManager
    private var focusRequest: AudioFocusRequestCompat? = null
    private var resumeOnFocusGain = false
    private var noisyRegistered = false
    private var coverKey: String? = null
    private var coverBitmap: Bitmap? = null

    /** Kopfhörer abgezogen → pausieren. */
    private val noisy = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            if (intent.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) emit("pause")
        }
    }

    private fun emit(action: String, positionMs: Long = 0) {
        AmberMobilePlugin.instance?.emit(action, positionMs)
    }

    override fun onCreate() {
        super.onCreate()
        running = true
        audio = getSystemService(Context.AUDIO_SERVICE) as AudioManager
        createChannel()
        session = MediaSessionCompat(this, "AmberPlayer").apply {
            setCallback(object : MediaSessionCompat.Callback() {
                override fun onPlay() = emit("play")
                override fun onPause() = emit("pause")
                override fun onSkipToNext() = emit("next")
                override fun onSkipToPrevious() = emit("prev")
                override fun onStop() = emit("pause")
                override fun onSeekTo(pos: Long) = emit("seek", pos)
            })
            isActive = true
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            "play" -> emit("play")
            "pause" -> emit("pause")
            "next" -> emit("next")
            "prev" -> emit("prev")
            Intent.ACTION_MEDIA_BUTTON -> MediaButtonReceiver.handleIntent(session, intent)
        }
        render(last ?: MediaArgs())
        return START_NOT_STICKY
    }

    private fun createChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(CHANNEL, getString(R.string.amber_channel), NotificationManager.IMPORTANCE_LOW)
            channel.description = getString(R.string.amber_channel_desc)
            channel.setShowBadge(false)
            getSystemService(NotificationManager::class.java).createNotificationChannel(channel)
        }
    }

    private fun action(name: String, code: Int): PendingIntent =
        PendingIntent.getService(
            this, code,
            Intent(this, PlaybackService::class.java).setAction(name),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )

    private fun cover(args: MediaArgs): Bitmap? {
        val data = args.cover ?: return null
        if (data.isEmpty()) return null
        val key = args.title + "|" + args.artist + "|" + data.length
        if (key != coverKey) {
            coverKey = key
            coverBitmap = try {
                val bytes = Base64.decode(data, Base64.DEFAULT)
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
            } catch (e: Exception) {
                null
            }
        }
        return coverBitmap
    }

    private fun render(args: MediaArgs) {
        val art = cover(args)
        session.setMetadata(
            MediaMetadataCompat.Builder()
                .putString(MediaMetadataCompat.METADATA_KEY_TITLE, args.title)
                .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, args.artist)
                .putString(MediaMetadataCompat.METADATA_KEY_ALBUM, args.album)
                .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, args.durationMs)
                .putBitmap(MediaMetadataCompat.METADATA_KEY_ALBUM_ART, art)
                .build()
        )
        session.setPlaybackState(
            PlaybackStateCompat.Builder()
                .setActions(
                    PlaybackStateCompat.ACTION_PLAY or PlaybackStateCompat.ACTION_PAUSE or
                        PlaybackStateCompat.ACTION_PLAY_PAUSE or PlaybackStateCompat.ACTION_SKIP_TO_NEXT or
                        PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS or PlaybackStateCompat.ACTION_SEEK_TO or
                        PlaybackStateCompat.ACTION_STOP
                )
                .setState(
                    if (args.playing) PlaybackStateCompat.STATE_PLAYING else PlaybackStateCompat.STATE_PAUSED,
                    args.positionMs,
                    if (args.playing) 1f else 0f,
                )
                .build()
        )

        val open = packageManager.getLaunchIntentForPackage(packageName)?.let {
            PendingIntent.getActivity(this, 0, it, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT)
        }
        val playPause = if (args.playing)
            NotificationCompat.Action(android.R.drawable.ic_media_pause, getString(R.string.amber_pause), action("pause", 2))
        else
            NotificationCompat.Action(android.R.drawable.ic_media_play, getString(R.string.amber_play), action("play", 2))

        val notification = NotificationCompat.Builder(this, CHANNEL)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentTitle(args.title.ifEmpty { "Amber Player" })
            .setContentText(args.artist)
            .setSubText(args.album)
            .setLargeIcon(art)
            .setContentIntent(open)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setOnlyAlertOnce(true)
            .setOngoing(args.playing)
            .addAction(android.R.drawable.ic_media_previous, getString(R.string.amber_prev), action("prev", 1))
            .addAction(playPause)
            .addAction(android.R.drawable.ic_media_next, getString(R.string.amber_next), action("next", 3))
            .setStyle(
                androidx.media.app.NotificationCompat.MediaStyle()
                    .setMediaSession(session.sessionToken)
                    .setShowActionsInCompactView(0, 1, 2)
            )
            .build()

        // Erst in den Vordergrund (Pflicht nach startForegroundService), bei Pause wieder lösen,
        // damit Android die App bei Bedarf beenden darf; die Benachrichtigung bleibt sichtbar.
        val type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK else 0
        ServiceCompat.startForeground(this, NOTIFICATION_ID, notification, type)
        if (args.playing) {
            requestFocus()
            registerNoisy()
        } else {
            ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_DETACH)
            unregisterNoisy()
        }
    }

    private fun requestFocus() {
        if (focusRequest != null) return
        val request = AudioFocusRequestCompat.Builder(AudioManagerCompat.AUDIOFOCUS_GAIN)
            .setAudioAttributes(
                AudioAttributesCompat.Builder()
                    .setUsage(AudioAttributesCompat.USAGE_MEDIA)
                    .setContentType(AudioAttributesCompat.CONTENT_TYPE_MUSIC)
                    .build()
            )
            .setOnAudioFocusChangeListener { change ->
                when (change) {
                    AudioManager.AUDIOFOCUS_LOSS -> {
                        resumeOnFocusGain = false
                        emit("pause")
                    }
                    // Anruf, Navigationsansage usw.: pausieren und danach weiterspielen
                    AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> {
                        resumeOnFocusGain = true
                        emit("pause")
                    }
                    AudioManager.AUDIOFOCUS_GAIN -> if (resumeOnFocusGain) {
                        resumeOnFocusGain = false
                        emit("play")
                    }
                }
            }
            .build()
        if (AudioManagerCompat.requestAudioFocus(audio, request) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED) {
            focusRequest = request
        }
    }

    private fun registerNoisy() {
        if (noisyRegistered) return
        ContextCompat.registerReceiver(this, noisy, IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY), ContextCompat.RECEIVER_NOT_EXPORTED)
        noisyRegistered = true
    }

    private fun unregisterNoisy() {
        if (!noisyRegistered) return
        unregisterReceiver(noisy)
        noisyRegistered = false
    }

    /** App aus der Übersicht gewischt: Musik beenden statt unsteuerbar weiterzuspielen. */
    override fun onTaskRemoved(rootIntent: Intent?) {
        stopSelf()
        android.os.Process.killProcess(android.os.Process.myPid())
    }

    override fun onDestroy() {
        unregisterNoisy()
        focusRequest?.let { AudioManagerCompat.abandonAudioFocusRequest(audio, it) }
        focusRequest = null
        session.isActive = false
        session.release()
        running = false
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
