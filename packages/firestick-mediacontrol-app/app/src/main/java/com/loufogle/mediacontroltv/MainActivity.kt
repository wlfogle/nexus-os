package com.loufogle.mediacontroltv

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout
import com.google.android.material.button.MaterialButton
import com.google.android.material.textfield.TextInputEditText
import com.loufogle.mediacontroltv.model.Service
import com.loufogle.mediacontroltv.model.ServiceListItem
import com.loufogle.mediacontroltv.network.ApiClient
import com.loufogle.mediacontroltv.ui.ServiceAdapter
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.text.DecimalFormat

class MainActivity : AppCompatActivity() {
    private lateinit var swipeRefresh: SwipeRefreshLayout
    private lateinit var recyclerView: RecyclerView
    private lateinit var txtStats: TextView
    private lateinit var txtSystem: TextView
    private lateinit var txtServer: TextView
    private lateinit var btnSettings: MaterialButton
    private lateinit var btnRefresh: MaterialButton
    private lateinit var btnWebDashboard: MaterialButton

    private lateinit var adapter: ServiceAdapter

    private val prefs by lazy { getSharedPreferences("msc_prefs", MODE_PRIVATE) }
    private val api by lazy { ApiClient { getServerUrl() } }

    private var autoRefreshJob: Job? = null
    private val pctFmt = DecimalFormat("0.0")

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        swipeRefresh = findViewById(R.id.swipeRefresh)
        recyclerView = findViewById(R.id.recyclerView)
        txtStats = findViewById(R.id.txtStats)
        txtSystem = findViewById(R.id.txtSystem)
        txtServer = findViewById(R.id.txtServer)
        btnSettings = findViewById(R.id.btnSettings)
        btnRefresh = findViewById(R.id.btnRefresh)
        btnWebDashboard = findViewById(R.id.btnWebDashboard)

        adapter = ServiceAdapter(
            onAction = { service, action -> performAction(service, action) },
            onLogs = { service -> showLogs(service) },
            onOpenUi = { service -> openServiceUi(service) }
        )

        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.adapter = adapter

        swipeRefresh.setOnRefreshListener { refreshAll() }
        btnRefresh.setOnClickListener { refreshAll() }
        btnSettings.setOnClickListener { openServerSettings() }
        btnWebDashboard.setOnClickListener {
            startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(getServerUrl())))
        }

        txtServer.text = getString(R.string.server_label, getServerUrl())
        refreshAll()
    }

    override fun onResume() {
        super.onResume()
        startAutoRefresh()
    }

    override fun onPause() {
        super.onPause()
        autoRefreshJob?.cancel()
    }

    private fun startAutoRefresh() {
        autoRefreshJob?.cancel()
        autoRefreshJob = lifecycleScope.launch {
            while (true) {
                delay(8000)
                refreshAll(showSpinner = false)
            }
        }
    }

    private fun refreshAll(showSpinner: Boolean = true) {
        lifecycleScope.launch {
            if (showSpinner) swipeRefresh.isRefreshing = true
            try {
                val services = api.getServices()
                val system = api.getSystem()

                val items = mutableListOf<ServiceListItem>()
                services.groups.forEach { group ->
                    items += ServiceListItem.Header("${group.category} (${group.services.size})")
                    items += group.services.map { ServiceListItem.Row(it) }
                }
                adapter.submit(items)

                txtStats.text = getString(
                    R.string.stats_line,
                    services.stats.running,
                    services.stats.stopped,
                    services.stats.total
                )
                txtSystem.text = getString(
                    R.string.system_line,
                    pctFmt.format(system.cpu.percent),
                    pctFmt.format(system.memory.percent),
                    pctFmt.format(system.disk.percent)
                )
                txtServer.text = getString(R.string.server_label, getServerUrl())
            } catch (e: Exception) {
                Toast.makeText(this@MainActivity, "Refresh failed: ${e.message}", Toast.LENGTH_LONG).show()
            } finally {
                swipeRefresh.isRefreshing = false
            }
        }
    }

    private fun performAction(service: Service, action: String) {
        lifecycleScope.launch {
            swipeRefresh.isRefreshing = true
            try {
                val ok = api.doAction(service.name, action)
                if (ok) {
                    Toast.makeText(
                        this@MainActivity,
                        "${service.display_name}: $action OK",
                        Toast.LENGTH_SHORT
                    ).show()
                } else {
                    Toast.makeText(
                        this@MainActivity,
                        "${service.display_name}: $action failed",
                        Toast.LENGTH_LONG
                    ).show()
                }
            } catch (e: Exception) {
                Toast.makeText(
                    this@MainActivity,
                    "${service.display_name}: ${e.message}",
                    Toast.LENGTH_LONG
                ).show()
            } finally {
                refreshAll(showSpinner = false)
            }
        }
    }

    private fun showLogs(service: Service) {
        lifecycleScope.launch {
            val logView = layoutInflater.inflate(R.layout.dialog_logs, null)
            val txtLogs = logView.findViewById<TextView>(R.id.txtLogs)
            val btnReload = logView.findViewById<MaterialButton>(R.id.btnReloadLogs)
            val btnMore = logView.findViewById<MaterialButton>(R.id.btnMoreLogs)
            var lines = 300

            val dialog = AlertDialog.Builder(this@MainActivity)
                .setTitle("Logs • ${service.display_name}")
                .setView(logView)
                .setNegativeButton("Close", null)
                .create()

            suspend fun loadLogs() {
                txtLogs.text = "Loading..."
                try {
                    txtLogs.text = api.getLogs(service.name, lines).ifBlank { "(no logs)" }
                } catch (e: Exception) {
                    txtLogs.text = "Failed to load logs: ${e.message}"
                }
            }

            btnReload.setOnClickListener { lifecycleScope.launch { loadLogs() } }
            btnMore.setOnClickListener {
                lines = (lines + 500).coerceAtMost(5000)
                lifecycleScope.launch { loadLogs() }
            }

            dialog.show()
            loadLogs()
        }
    }

    private fun openServiceUi(service: Service) {
        val raw = service.web_url ?: return
        val resolved = rewriteLocalhostForRemote(raw)
        startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(resolved)))
    }

    private fun rewriteLocalhostForRemote(url: String): String {
        val target = Uri.parse(url)
        if (target.host != "localhost" && target.host != "127.0.0.1") return url

        val server = Uri.parse(getServerUrl())
        val host = server.host ?: return url
        val scheme = server.scheme ?: "http"
        val portPart = if (target.port > 0) ":${target.port}" else ""
        val path = target.encodedPath ?: ""
        val query = target.encodedQuery?.let { "?$it" } ?: ""
        return "$scheme://$host$portPart$path$query"
    }

    private fun openServerSettings() {
        val input = TextInputEditText(this).apply {
            setText(getServerUrl())
            setSingleLine(true)
        }
        AlertDialog.Builder(this)
            .setTitle("MediaStack Server URL")
            .setView(input)
            .setPositiveButton("Save") { _, _ ->
                val value = input.text?.toString()?.trim().orEmpty()
                if (value.isBlank()) return@setPositiveButton
                prefs.edit().putString("server_url", value).apply()
                txtServer.text = getString(R.string.server_label, getServerUrl())
                refreshAll()
            }
            .setNeutralButton("Reset") { _, _ ->
                prefs.edit().putString("server_url", BuildConfig.MEDIA_CONTROL_URL).apply()
                txtServer.text = getString(R.string.server_label, getServerUrl())
                refreshAll()
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun getServerUrl(): String {
        return prefs.getString("server_url", BuildConfig.MEDIA_CONTROL_URL)
            ?.trim()
            ?.ifBlank { BuildConfig.MEDIA_CONTROL_URL }
            ?: BuildConfig.MEDIA_CONTROL_URL
    }
}
