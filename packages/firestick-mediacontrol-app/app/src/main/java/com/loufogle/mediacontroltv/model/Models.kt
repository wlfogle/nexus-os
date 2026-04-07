package com.loufogle.mediacontroltv.model

data class Service(
    val name: String,
    val display_name: String,
    val icon: String,
    val category: String,
    val status: String,
    val health: String?,
    val started_at: String?,
    val image: String?,
    val ports: List<String> = emptyList(),
    val web_url: String?
)

data class ServiceGroup(
    val category: String,
    val services: List<Service>
)

data class ServiceStats(
    val total: Int,
    val running: Int,
    val stopped: Int
)

data class ServicesResponse(
    val groups: List<ServiceGroup>,
    val stats: ServiceStats
)

data class CpuStats(
    val percent: Double,
    val cores: Int
)

data class MemoryStats(
    val total: Long,
    val used: Long,
    val percent: Double
)

data class DiskStats(
    val total: Long,
    val used: Long,
    val free: Long,
    val percent: Double
)

data class SystemResponse(
    val cpu: CpuStats,
    val memory: MemoryStats,
    val disk: DiskStats
)

sealed class ServiceListItem {
    data class Header(val title: String) : ServiceListItem()
    data class Row(val service: Service) : ServiceListItem()
}
