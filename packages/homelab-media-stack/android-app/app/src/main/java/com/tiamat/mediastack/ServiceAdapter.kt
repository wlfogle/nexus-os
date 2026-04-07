package com.tiamat.mediastack

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.RecyclerView
import com.tiamat.mediastack.databinding.ItemServiceBinding

class ServiceAdapter(
    private var services: List<MediaService>,
    private val onClick: (MediaService) -> Unit
) : RecyclerView.Adapter<ServiceAdapter.ServiceViewHolder>() {

    inner class ServiceViewHolder(
        private val binding: ItemServiceBinding
    ) : RecyclerView.ViewHolder(binding.root) {

        fun bind(service: MediaService) {
            binding.serviceName.text        = service.name
            binding.serviceDescription.text = service.description
            binding.serviceIcon.setImageResource(service.iconResId)

            // Coming Soon overlay for unavailable services
            binding.comingSoonOverlay.visibility =
                if (service.available) View.GONE else View.VISIBLE

            if (service.available) {
                binding.root.alpha = 1.0f
                binding.root.setOnClickListener { onClick(service) }
            } else {
                binding.root.alpha = 0.5f
                binding.root.setOnClickListener(null)
                binding.root.isClickable = false
            }

            // D-pad focus highlight (Fire TV)
            binding.root.setOnFocusChangeListener { v, hasFocus ->
                if (service.available) {
                    v.scaleX = if (hasFocus) 1.05f else 1.0f
                    v.scaleY = if (hasFocus) 1.05f else 1.0f
                    v.elevation = if (hasFocus) 16f else 4f
                }
            }

            binding.root.isFocusable = service.available
            binding.root.isFocusableInTouchMode = service.available
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ServiceViewHolder {
        val binding = ItemServiceBinding.inflate(
            LayoutInflater.from(parent.context), parent, false
        )
        return ServiceViewHolder(binding)
    }

    override fun onBindViewHolder(holder: ServiceViewHolder, position: Int) {
        holder.bind(services[position])
    }

    override fun getItemCount(): Int = services.size
}
