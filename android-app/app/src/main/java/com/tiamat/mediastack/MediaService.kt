package com.tiamat.mediastack

/**
 * Represents a content service shown in the dashboard.
 * The app is a search-and-add interface — the arr stack + qBittorrent
 * handle downloading, and Jellyfin/Plex auto-import finished media.
 */
data class MediaService(
    val name:        String,
    val url:         String,
    val description: String,
    val iconResId:   Int,
    val available:   Boolean = true
)

/**
 * Service list — per-service LXC architecture on Tiamat (192.168.12.242).
 *
 * Available services have static or known DHCP IPs.
 * Unavailable services are planned but not yet deployed (need Tiamat RAM upgrade).
 * Update URLs here when new containers come online.
 */
object ServiceRepository {

    fun getServices(): List<MediaService> = listOf(

        MediaService(
            name        = "Movies & TV",
            url         = "http://192.168.12.151:5055",
            description = "Search & request via Jellyseerr",
            iconResId   = R.drawable.ic_service_overseerr,
            available   = true
        ),

        MediaService(
            name        = "Books",
            url         = "http://192.168.12.217:8787",
            description = "Search & add via Readarr",
            iconResId   = R.drawable.ic_service_readarr,
            available   = false  // CT-217 planned (static IP) — not created yet
        ),

        MediaService(
            name        = "Music",
            url         = "",
            description = "Search & add via Lidarr",
            iconResId   = R.drawable.ic_service_lidarr,
            available   = false  // CT not deployed — needs Tiamat RAM upgrade
        ),

        MediaService(
            name        = "Audiobooks",
            url         = "",
            description = "Browse & add via Audiobookshelf",
            iconResId   = R.drawable.ic_service_audiobookshelf,
            available   = false  // CT-232 planned, DHCP — not deployed yet
        ),

        MediaService(
            name        = "eBooks",
            url         = "",
            description = "Browse & add via Calibre-Web",
            iconResId   = R.drawable.ic_service_calibreweb,
            available   = false  // CT-233 planned, DHCP — not deployed yet
        ),

        MediaService(
            name        = "Add Torrent",
            url         = "http://192.168.12.212:8080",
            description = "Paste magnet or torrent URL",
            iconResId   = R.drawable.ic_service_qbit,
            available   = true
        )
    )
}
