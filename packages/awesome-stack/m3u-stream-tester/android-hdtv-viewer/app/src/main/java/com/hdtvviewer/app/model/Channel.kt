package com.hdtvviewer.app.model

data class Channel(
    val name: String,
    val url: String,
    val group: String? = null,
    val logo: String? = null,
    val tvgId: String? = null
)
