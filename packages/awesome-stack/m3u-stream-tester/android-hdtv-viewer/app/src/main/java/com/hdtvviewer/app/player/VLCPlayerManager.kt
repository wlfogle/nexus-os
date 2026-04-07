package com.hdtvviewer.app.player

import android.content.Context
import android.net.Uri
import android.view.SurfaceView
import org.videolan.libvlc.LibVLC
import org.videolan.libvlc.Media
import org.videolan.libvlc.MediaPlayer
import org.videolan.libvlc.util.VLCVideoLayout

class VLCPlayerManager(private val context: Context) {
    private var libVLC: LibVLC? = null
    private var mediaPlayer: MediaPlayer? = null
    private var currentUrl: String? = null
    
    var onStateChanged: ((PlayerState) -> Unit)? = null
    var onError: ((String) -> Unit)? = null
    
    enum class PlayerState {
        IDLE, LOADING, PLAYING, PAUSED, STOPPED, ERROR
    }
    
    private var currentState = PlayerState.IDLE
        set(value) {
            field = value
            onStateChanged?.invoke(value)
        }
    
    init {
        initializeVLC()
    }
    
    private fun initializeVLC() {
        try {
            val options = arrayListOf(
                "--aout=opensles",
                "--audio-time-stretch",
                "--auto-adjust-pts-delay",
                "--avcodec-skiploopfilter", "4",
                "--avcodec-skip-frame", "0",
                "--avcodec-skip-idct", "0",
                "--subsdec-encoding", "UTF-8",
                "--stats",
                "--network-caching", "1500",
                "--live-caching", "1500",
                "--clock-jitter", "0",
                "--clock-synchro", "0"
            )
            
            libVLC = LibVLC(context, options)
            mediaPlayer = MediaPlayer(libVLC).apply {
                setEventListener { event ->
                    when (event.type) {
                        MediaPlayer.Event.Opening -> {
                            currentState = PlayerState.LOADING
                        }
                        MediaPlayer.Event.Playing -> {
                            currentState = PlayerState.PLAYING
                        }
                        MediaPlayer.Event.Paused -> {
                            currentState = PlayerState.PAUSED
                        }
                        MediaPlayer.Event.Stopped -> {
                            currentState = PlayerState.STOPPED
                        }
                        MediaPlayer.Event.EncounteredError -> {
                            currentState = PlayerState.ERROR
                            onError?.invoke("Playback error occurred")
                        }
                        MediaPlayer.Event.EndReached -> {
                            currentState = PlayerState.STOPPED
                        }
                    }
                }
            }
        } catch (e: Exception) {
            onError?.invoke("Failed to initialize VLC: ${e.message}")
        }
    }
    
    fun attachSurface(surfaceView: SurfaceView) {
        try {
            mediaPlayer?.vlcVout?.setVideoView(surfaceView)
            mediaPlayer?.vlcVout?.attachViews()
        } catch (e: Exception) {
            onError?.invoke("Failed to attach surface: ${e.message}")
        }
    }
    
    fun detachSurface() {
        try {
            mediaPlayer?.vlcVout?.detachViews()
        } catch (e: Exception) {
            // Ignore detach errors
        }
    }
    
    fun playUrl(url: String) {
        try {
            currentUrl = url
            val media = Media(libVLC, Uri.parse(url))
            mediaPlayer?.media = media
            mediaPlayer?.play()
        } catch (e: Exception) {
            onError?.invoke("Failed to play URL: ${e.message}")
            currentState = PlayerState.ERROR
        }
    }
    
    fun play() {
        try {
            if (currentState == PlayerState.PAUSED) {
                mediaPlayer?.play()
            } else {
                currentUrl?.let { playUrl(it) }
            }
        } catch (e: Exception) {
            onError?.invoke("Failed to play: ${e.message}")
        }
    }
    
    fun pause() {
        try {
            mediaPlayer?.pause()
        } catch (e: Exception) {
            onError?.invoke("Failed to pause: ${e.message}")
        }
    }
    
    fun stop() {
        try {
            mediaPlayer?.stop()
            currentState = PlayerState.STOPPED
        } catch (e: Exception) {
            onError?.invoke("Failed to stop: ${e.message}")
        }
    }
    
    fun isPlaying(): Boolean = currentState == PlayerState.PLAYING
    
    fun getState(): PlayerState = currentState
    
    fun setVolume(volume: Int) {
        try {
            mediaPlayer?.setVolume(volume.coerceIn(0, 200))
        } catch (e: Exception) {
            onError?.invoke("Failed to set volume: ${e.message}")
        }
    }
    
    fun release() {
        try {
            mediaPlayer?.release()
            libVLC?.release()
            currentState = PlayerState.IDLE
        } catch (e: Exception) {
            // Ignore release errors
        }
    }
}
