package com.hdtvviewer.app.network

import com.hdtvviewer.app.model.Channel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.logging.HttpLoggingInterceptor
import java.util.concurrent.TimeUnit

class PlaylistService {
    
    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .addInterceptor(HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BASIC
        })
        .build()
    
    private val parser = M3UParser()
    
    suspend fun fetchPlaylist(url: String): Result<List<Channel>> {
        return withContext(Dispatchers.IO) {
            try {
                val request = Request.Builder()
                    .url(url)
                    .addHeader("User-Agent", "HDTVViewer/1.0")
                    .build()
                
                client.newCall(request).execute().use { response ->
                    if (!response.isSuccessful) {
                        return@withContext Result.failure(
                            Exception("Failed to fetch playlist: ${response.code}")
                        )
                    }
                    
                    val content = response.body?.string() ?: ""
                    if (content.isEmpty()) {
                        return@withContext Result.failure(
                            Exception("Empty playlist content")
                        )
                    }
                    
                    val channels = parser.parseM3U(content)
                    if (channels.isEmpty()) {
                        return@withContext Result.failure(
                            Exception("No channels found in playlist")
                        )
                    }
                    
                    Result.success(channels)
                }
            } catch (e: Exception) {
                Result.failure(e)
            }
        }
    }
}
