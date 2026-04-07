package com.hdtvviewer.app.network

import com.hdtvviewer.app.model.Channel
import java.io.BufferedReader
import java.io.StringReader

class M3UParser {
    
    fun parseM3U(content: String): List<Channel> {
        val channels = mutableListOf<Channel>()
        val reader = BufferedReader(StringReader(content))
        var line: String?
        var currentChannel: Channel? = null
        
        while (reader.readLine().also { line = it } != null) {
            line?.let { currentLine ->
                when {
                    currentLine.startsWith("#EXTM3U") -> {
                        // M3U header, continue
                    }
                    currentLine.startsWith("#EXTINF:") -> {
                        // Parse channel info
                        currentChannel = parseExtInf(currentLine)
                    }
                    currentLine.startsWith("http") || currentLine.startsWith("https") -> {
                        // Channel URL
                        currentChannel?.let { channel ->
                            val completeChannel = channel.copy(url = currentLine.trim())
                            channels.add(completeChannel)
                        }
                        currentChannel = null
                    }
                }
            }
        }
        
        return channels
    }
    
    private fun parseExtInf(line: String): Channel {
        // Parse #EXTINF:-1 tvg-id="..." tvg-name="..." tvg-logo="..." group-title="...",Channel Name
        var name = "Unknown Channel"
        var group: String? = null
        var logo: String? = null
        var tvgId: String? = null
        
        // Extract attributes
        val attributePattern = """(\w+(?:-\w+)*)="([^"]*)"""".toRegex()
        val matches = attributePattern.findAll(line)
        
        for (match in matches) {
            val key = match.groupValues[1]
            val value = match.groupValues[2]
            
            when (key.lowercase()) {
                "tvg-id" -> tvgId = value
                "tvg-name" -> name = value
                "tvg-logo" -> logo = value
                "group-title" -> group = value
            }
        }
        
        // Extract channel name from the end of the line
        val commaIndex = line.lastIndexOf(',')
        if (commaIndex != -1 && commaIndex < line.length - 1) {
            val extractedName = line.substring(commaIndex + 1).trim()
            if (extractedName.isNotEmpty()) {
                name = extractedName
            }
        }
        
        return Channel(
            name = name,
            url = "", // Will be set when URL line is parsed
            group = group,
            logo = logo,
            tvgId = tvgId
        )
    }
}
