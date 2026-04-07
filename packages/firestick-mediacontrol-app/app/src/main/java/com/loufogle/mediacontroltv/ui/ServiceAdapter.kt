package com.loufogle.mediacontroltv.ui

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import com.loufogle.mediacontroltv.R
import com.loufogle.mediacontroltv.model.Service
import com.loufogle.mediacontroltv.model.ServiceListItem

class ServiceAdapter(
    private val onAction: (Service, String) -> Unit,
    private val onLogs: (Service) -> Unit,
    private val onOpenUi: (Service) -> Unit
) : RecyclerView.Adapter<RecyclerView.ViewHolder>() {

    private val items = mutableListOf<ServiceListItem>()

    fun submit(newItems: List<ServiceListItem>) {
        items.clear()
        items.addAll(newItems)
        notifyDataSetChanged()
    }

    override fun getItemViewType(position: Int): Int {
        return when (items[position]) {
            is ServiceListItem.Header -> 0
            is ServiceListItem.Row -> 1
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RecyclerView.ViewHolder {
        return if (viewType == 0) {
            val v = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_section_header, parent, false)
            HeaderVH(v)
        } else {
            val v = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_service, parent, false)
            ServiceVH(v, onAction, onLogs, onOpenUi)
        }
    }

    override fun onBindViewHolder(holder: RecyclerView.ViewHolder, position: Int) {
        when (val item = items[position]) {
            is ServiceListItem.Header -> (holder as HeaderVH).bind(item.title)
            is ServiceListItem.Row -> (holder as ServiceVH).bind(item.service)
        }
    }

    override fun getItemCount(): Int = items.size

    private class HeaderVH(view: View) : RecyclerView.ViewHolder(view) {
        private val title: TextView = view.findViewById(R.id.sectionTitle)
        fun bind(text: String) {
            title.text = text
        }
    }

    private class ServiceVH(
        view: View,
        private val onAction: (Service, String) -> Unit,
        private val onLogs: (Service) -> Unit,
        private val onOpenUi: (Service) -> Unit
    ) : RecyclerView.ViewHolder(view) {
        private val name: TextView = view.findViewById(R.id.serviceName)
        private val status: TextView = view.findViewById(R.id.serviceStatus)
        private val image: TextView = view.findViewById(R.id.serviceImage)
        private val ports: TextView = view.findViewById(R.id.servicePorts)
        private val btnStart: Button = view.findViewById(R.id.btnStart)
        private val btnStop: Button = view.findViewById(R.id.btnStop)
        private val btnRestart: Button = view.findViewById(R.id.btnRestart)
        private val btnLogs: Button = view.findViewById(R.id.btnLogs)
        private val btnOpen: Button = view.findViewById(R.id.btnOpen)

        fun bind(service: Service) {
            name.text = "${service.icon} ${service.display_name}"
            val healthSuffix = if (!service.health.isNullOrBlank()) " (${service.health})" else ""
            status.text = "Status: ${service.status}$healthSuffix"
            image.text = "Image: ${service.image ?: "unknown"}"
            ports.text = if (service.ports.isEmpty()) "Ports: none" else "Ports: ${service.ports.joinToString(", ")}"

            val running = service.status == "running"
            btnStart.isEnabled = !running
            btnStop.isEnabled = running
            btnRestart.isEnabled = running
            btnOpen.isEnabled = running && !service.web_url.isNullOrBlank()

            btnStart.setOnClickListener { onAction(service, "start") }
            btnStop.setOnClickListener { onAction(service, "stop") }
            btnRestart.setOnClickListener { onAction(service, "restart") }
            btnLogs.setOnClickListener { onLogs(service) }
            btnOpen.setOnClickListener { onOpenUi(service) }
        }
    }
}
