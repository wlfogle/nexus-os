package com.loufogle.mediacontroltv.network

import com.google.gson.Gson
import com.loufogle.mediacontroltv.model.ServicesResponse
import com.loufogle.mediacontroltv.model.SystemResponse
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.concurrent.TimeUnit

class ApiClient(
    private val baseUrlProvider: () -> String
) {
    private val gson = Gson()
    private val jsonMediaType = "application/json".toMediaType()
    private val client = OkHttpClient.Builder()
        .connectTimeout(12, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(20, TimeUnit.SECONDS)
        .build()

    private fun endpoint(path: String): String {
        return "${baseUrlProvider().trimEnd('/')}$path"
    }

    suspend fun getServices(): ServicesResponse = withContext(Dispatchers.IO) {
        val req = Request.Builder()
            .url(endpoint("/api/services"))
            .get()
            .build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) throw RuntimeException("Services API failed: HTTP ${resp.code}")
            val body = resp.body?.string() ?: throw RuntimeException("Services API returned empty body")
            gson.fromJson(body, ServicesResponse::class.java)
        }
    }

    suspend fun getSystem(): SystemResponse = withContext(Dispatchers.IO) {
        val req = Request.Builder()
            .url(endpoint("/api/system"))
            .get()
            .build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) throw RuntimeException("System API failed: HTTP ${resp.code}")
            val body = resp.body?.string() ?: throw RuntimeException("System API returned empty body")
            gson.fromJson(body, SystemResponse::class.java)
        }
    }

    suspend fun doAction(serviceName: String, action: String): Boolean = withContext(Dispatchers.IO) {
        val req = Request.Builder()
            .url(endpoint("/api/services/$serviceName/$action"))
            .post("{}".toRequestBody(jsonMediaType))
            .build()
        client.newCall(req).execute().use { resp ->
            resp.isSuccessful
        }
    }

    suspend fun getLogs(serviceName: String, lines: Int = 300): String = withContext(Dispatchers.IO) {
        val req = Request.Builder()
            .url(endpoint("/api/services/$serviceName/logs?lines=$lines"))
            .get()
            .build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) throw RuntimeException("Logs API failed: HTTP ${resp.code}")
            val body = resp.body?.string() ?: "{}"
            gson.fromJson(body, Map::class.java)["logs"]?.toString().orEmpty()
        }
    }

    suspend fun checkHealth(): Boolean = withContext(Dispatchers.IO) {
        val req = Request.Builder()
            .url(endpoint("/api/health"))
            .get()
            .build()
        client.newCall(req).execute().use { resp -> resp.isSuccessful }
    }
}
