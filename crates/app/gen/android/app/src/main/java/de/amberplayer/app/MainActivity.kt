package de.amberplayer.app

import android.content.Intent
import android.os.Bundle
import android.webkit.WebView
import androidx.lifecycle.Lifecycle
import androidx.webkit.WebViewCompat
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : TauriActivity() {
  /**
   * Beendet Android den Prozess der Oberfläche (Speichermangel, meist bei gesperrtem Bildschirm),
   * würde die WebView sonst die ganze App mitreißen – samt laufender Musik. Stattdessen wird nur
   * die Oberfläche neu aufgebaut; Player, Benachrichtigung und Medientasten laufen weiter.
   */
  override fun onWebViewCreate(webView: WebView) {
    val inner = WebViewCompat.getWebViewClient(webView)
    webView.webViewClient = RecoveringWebViewClient(inner) {
      de.amberplayer.mobile.AmberMobilePlugin.instance?.note("Oberfläche wurde von Android beendet – wird neu geladen")
      val visible = lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)
      finish()
      if (visible) startActivity(Intent(this, MainActivity::class.java))
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // Abstand zu allem, was gerade sichtbar ist: Kamera-Aussparung und – wenn das Vollbild
    // aus ist – Status- und Navigationsleiste. Ausgeblendete Leisten zählen hier als 0.
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      insets
    }
    applyFullscreen(this)
  }

  // Nach Dialogen, App-Wechsel usw. den gewählten Zustand wiederherstellen.
  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus) applyFullscreen(this)
  }

  companion object {
    private const val PREFS = "amber_prefs"
    private const val KEY = "fullscreen"

    fun isFullscreen(activity: android.app.Activity): Boolean =
      activity.getSharedPreferences(PREFS, MODE_PRIVATE).getBoolean(KEY, true)

    fun setFullscreen(activity: android.app.Activity, on: Boolean) {
      activity.getSharedPreferences(PREFS, MODE_PRIVATE).edit().putBoolean(KEY, on).apply()
      applyFullscreen(activity)
    }

    /** Vollbild an: Leisten aus, Wischen vom Rand zeigt sie kurz. Aus: Leisten immer sichtbar. */
    fun applyFullscreen(activity: android.app.Activity) {
      val controller = WindowCompat.getInsetsController(activity.window, activity.window.decorView)
      if (isFullscreen(activity)) {
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        controller.hide(WindowInsetsCompat.Type.systemBars())
      } else {
        controller.show(WindowInsetsCompat.Type.systemBars())
      }
    }
  }
}
