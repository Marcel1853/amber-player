package de.amberplayer.app

import android.graphics.Bitmap
import android.webkit.RenderProcessGoneDetail
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient

/**
 * Reicht alles an den WebView-Client von Tauri weiter und fängt nur einen Fall ab: Android hat
 * den Prozess der Oberfläche beendet. Ohne das würde die ganze App beendet.
 */
class RecoveringWebViewClient(
  private val inner: WebViewClient,
  private val onGone: () -> Unit,
) : WebViewClient() {
  override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? =
    inner.shouldInterceptRequest(view, request)

  override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean =
    inner.shouldOverrideUrlLoading(view, request)

  override fun onPageStarted(view: WebView, url: String, favicon: Bitmap?) = inner.onPageStarted(view, url, favicon)

  override fun onPageFinished(view: WebView, url: String) = inner.onPageFinished(view, url)

  override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) =
    inner.onReceivedError(view, request, error)

  override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean {
    view.post { onGone() }
    return true // die App läuft weiter
  }
}
