package com.tiamat.mediastack

import android.content.Intent
import android.os.Bundle
import android.view.KeyEvent
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.GridLayoutManager
import com.tiamat.mediastack.databinding.ActivityTvMainBinding

/**
 * Fire TV / Android TV leanback launcher.
 * D-pad-navigable grid of content services.
 * Each card opens WebViewActivity to search & add content.
 */
class TvMainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityTvMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityTvMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val services = ServiceRepository.getServices()
        val adapter = ServiceAdapter(services) { service ->
            startActivity(Intent(this, WebViewActivity::class.java).apply {
                putExtra(WebViewActivity.EXTRA_URL, service.url)
                putExtra(WebViewActivity.EXTRA_TITLE, service.name)
            })
        }

        binding.recyclerView.layoutManager = GridLayoutManager(this, 3)
        binding.recyclerView.adapter = adapter

        // Give the first available item D-pad focus
        val firstAvailable = services.indexOfFirst { it.available }
        if (firstAvailable >= 0) {
            binding.recyclerView.post {
                binding.recyclerView.findViewHolderForAdapterPosition(firstAvailable)
                    ?.itemView?.requestFocus()
            }
        }
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        return when (keyCode) {
            KeyEvent.KEYCODE_BACK, KeyEvent.KEYCODE_HOME -> { finish(); true }
            else -> super.onKeyDown(keyCode, event)
        }
    }
}
