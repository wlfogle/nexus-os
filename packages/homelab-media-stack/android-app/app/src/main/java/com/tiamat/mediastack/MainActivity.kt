package com.tiamat.mediastack

import android.content.Intent
import android.os.Bundle
import android.view.Menu
import android.view.MenuItem
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.GridLayoutManager
import com.tiamat.mediastack.databinding.ActivityMainBinding

class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setSupportActionBar(binding.toolbar)
        supportActionBar?.title = getString(R.string.app_name)

        val services = ServiceRepository.getServices()
        val adapter = ServiceAdapter(services) { service ->
            startActivity(Intent(this, WebViewActivity::class.java).apply {
                putExtra(WebViewActivity.EXTRA_URL, service.url)
                putExtra(WebViewActivity.EXTRA_TITLE, service.name)
            })
        }

        val spanCount = if (resources.displayMetrics.widthPixels > 1200) 3 else 2
        binding.recyclerView.layoutManager = GridLayoutManager(this, spanCount)
        binding.recyclerView.adapter = adapter
    }

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        menuInflater.inflate(R.menu.main_menu, menu)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.menu_settings -> {
                val services = ServiceRepository.getServices()
                val info = services.joinToString("\n") { s ->
                    "${s.name}: ${if (s.available) s.url else "Not deployed"}"
                }
                AlertDialog.Builder(this)
                    .setTitle("Service URLs")
                    .setMessage("Tiamat @ 192.168.12.242\n\n$info\n\nEdit MediaService.kt to change URLs.")
                    .setPositiveButton("OK", null)
                    .show()
                true
            }
            R.id.menu_about -> {
                AlertDialog.Builder(this)
                    .setTitle("TiamatsStack")
                    .setMessage(
                        "Search & add content to your homelab.\n\n" +
                        "The arr stack downloads via qBittorrent\n" +
                        "and Jellyfin/Plex auto-import media.\n\n" +
                        "Tiamat @ 192.168.12.242\n" +
                        "v${BuildConfig.VERSION_NAME}"
                    )
                    .setPositiveButton("OK", null)
                    .show()
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }
}
