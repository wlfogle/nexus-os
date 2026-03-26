package com.tiamat.mediastack

import android.annotation.SuppressLint
import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import android.webkit.*
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.tiamat.mediastack.databinding.ActivityWebviewBinding

class WebViewActivity : AppCompatActivity() {

    companion object {
        const val EXTRA_URL   = "extra_url"
        const val EXTRA_TITLE = "extra_title"
    }

    private lateinit var binding: ActivityWebviewBinding

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityWebviewBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val url   = intent.getStringExtra(EXTRA_URL)   ?: "http://192.168.12.231:8096"
        val title = intent.getStringExtra(EXTRA_TITLE) ?: getString(R.string.app_name)

        setSupportActionBar(binding.toolbar)
        supportActionBar?.apply {
            setDisplayHomeAsUpEnabled(true)
            this.title = title
        }

        configureWebView()
        binding.webView.loadUrl(url)
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configureWebView() {
        binding.webView.apply {
            settings.apply {
                javaScriptEnabled         = true
                domStorageEnabled         = true
                loadWithOverviewMode      = true
                useWideViewPort           = true
                setSupportZoom(true)
                builtInZoomControls       = true
                displayZoomControls       = false
                mediaPlaybackRequiresUserGesture = false
                allowFileAccess           = true
                // Required for media-rich Jellyfin/Plex UIs
                mixedContentMode          = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            }

            webViewClient = object : WebViewClient() {
                override fun onPageStarted(view: WebView?, url: String?, favicon: android.graphics.Bitmap?) {
                    binding.progressBar.visibility = View.VISIBLE
                }
                override fun onPageFinished(view: WebView?, url: String?) {
                    binding.progressBar.visibility = View.GONE
                }
                override fun onReceivedError(
                    view: WebView?,
                    request: WebResourceRequest?,
                    error: WebResourceError?
                ) {
                    binding.progressBar.visibility = View.GONE
                    Toast.makeText(
                        this@WebViewActivity,
                        "Failed to load: ${error?.description}",
                        Toast.LENGTH_SHORT
                    ).show()
                }
            }

            webChromeClient = object : WebChromeClient() {
                override fun onProgressChanged(view: WebView?, newProgress: Int) {
                    binding.progressBar.progress = newProgress
                }
            }

            // Allow Fire TV D-pad to scroll/navigate the page
            isFocusable        = true
            isFocusableInTouchMode = true
            requestFocus()
        }
    }

    // ── Fire TV D-pad navigation ──────────────────────────────────────────────
    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        return when (keyCode) {
            // D-pad center / OK → click focused element
            KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER -> {
                binding.webView.evaluateJavascript(
                    "document.activeElement && document.activeElement.click()", null
                )
                true
            }
            // Back key: navigate WebView history before finishing activity
            KeyEvent.KEYCODE_BACK -> {
                if (binding.webView.canGoBack()) {
                    binding.webView.goBack()
                    true
                } else {
                    finish()
                    true
                }
            }
            // D-pad scroll
            KeyEvent.KEYCODE_DPAD_UP    -> { binding.webView.scrollBy(0, -150); true }
            KeyEvent.KEYCODE_DPAD_DOWN  -> { binding.webView.scrollBy(0,  150); true }
            KeyEvent.KEYCODE_DPAD_LEFT  -> { binding.webView.scrollBy(-150, 0); true }
            KeyEvent.KEYCODE_DPAD_RIGHT -> { binding.webView.scrollBy( 150, 0); true }
            else -> super.onKeyDown(keyCode, event)
        }
    }

    override fun onSupportNavigateUp(): Boolean {
        if (binding.webView.canGoBack()) {
            binding.webView.goBack()
        } else {
            finish()
        }
        return true
    }
}
