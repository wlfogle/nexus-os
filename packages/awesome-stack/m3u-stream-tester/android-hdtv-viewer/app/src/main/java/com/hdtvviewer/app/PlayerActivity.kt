package com.hdtvviewer.app

import android.content.pm.ActivityInfo
import android.net.Uri
import android.os.Bundle
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.hdtvviewer.app.ui.theme.HDTVViewerTheme
import org.videolan.libvlc.LibVLC
import org.videolan.libvlc.Media
import org.videolan.libvlc.MediaPlayer
import org.videolan.libvlc.util.VLCVideoLayout

class PlayerActivity : ComponentActivity() {
    private var libVLC: LibVLC? = null
    private var mediaPlayer: MediaPlayer? = null
    private var channelName: String = ""
    private var channelUrl: String = ""
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Set landscape orientation
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        
        // Get channel data from intent
        channelName = intent.getStringExtra("channel_name") ?: "Unknown Channel"
        channelUrl = intent.getStringExtra("channel_url") ?: ""
        
        // Initialize VLC
        try {
            libVLC = LibVLC(this, arrayListOf(
                "--aout=opensles",
                "--audio-time-stretch",
                "--auto-adjust-pts-delay",
                "--avcodec-skiploopfilter", "4",
                "--avcodec-skip-frame", "0",
                "--avcodec-skip-idct", "0",
                "--subsdec-encoding", "UTF-8",
                "--stats",
                "--network-caching=1000"
            ))
            mediaPlayer = MediaPlayer(libVLC)
        } catch (e: Exception) {
            e.printStackTrace()
            finish()
            return
        }
        
        setContent {
            HDTVViewerTheme {
                PlayerScreen(
                    channelName = channelName,
                    channelUrl = channelUrl,
                    onBackClick = { finish() },
                    onPlayClick = { playStream() },
                    onPauseClick = { pauseStream() },
                    onStopClick = { stopStream() }
                )
            }
        }
    }
    
    private fun playStream() {
        if (channelUrl.isNotBlank()) {
            try {
                val media = Media(libVLC, Uri.parse(channelUrl))
                mediaPlayer?.media = media
                mediaPlayer?.play()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }
    
    private fun pauseStream() {
        mediaPlayer?.pause()
    }
    
    private fun stopStream() {
        mediaPlayer?.stop()
    }
    
    override fun onDestroy() {
        super.onDestroy()
        mediaPlayer?.release()
        libVLC?.release()
    }
    
    override fun onPause() {
        super.onPause()
        mediaPlayer?.pause()
    }
    
    override fun onResume() {
        super.onResume()
        if (mediaPlayer?.isPlaying == false) {
            mediaPlayer?.play()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlayerScreen(
    channelName: String,
    channelUrl: String,
    onBackClick: () -> Unit,
    onPlayClick: () -> Unit,
    onPauseClick: () -> Unit,
    onStopClick: () -> Unit
) {
    var isPlaying by remember { mutableStateOf(false) }
    var showControls by remember { mutableStateOf(true) }
    
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
    ) {
        // Video Surface
        AndroidView(
            factory = { context ->
                SurfaceView(context).apply {
                    layoutParams = FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT
                    )
                    holder.addCallback(object : SurfaceHolder.Callback {
                        override fun surfaceCreated(holder: SurfaceHolder) {
                            // Surface created - VLC can attach to it
                        }
                        
                        override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
                            // Surface changed
                        }
                        
                        override fun surfaceDestroyed(holder: SurfaceHolder) {
                            // Surface destroyed
                        }
                    })
                }
            },
            modifier = Modifier.fillMaxSize()
        )
        
        // Top Controls
        if (showControls) {
            TopAppBar(
                title = {
                    Text(
                        text = channelName,
                        color = Color.White,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Medium
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onBackClick) {
                        Icon(
                            imageVector = Icons.Default.ArrowBack,
                            contentDescription = "Back",
                            tint = Color.White
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Color.Black.copy(alpha = 0.7f)
                ),
                modifier = Modifier.align(Alignment.TopCenter)
            )
        }
        
        // Bottom Controls
        if (showControls) {
            Card(
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .fillMaxWidth()
                    .padding(16.dp)
                    .clip(RoundedCornerShape(8.dp)),
                colors = CardDefaults.cardColors(
                    containerColor = Color.Black.copy(alpha = 0.7f)
                )
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    horizontalArrangement = Arrangement.SpaceEvenly,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(
                        onClick = {
                            if (isPlaying) {
                                onPauseClick()
                                isPlaying = false
                            } else {
                                onPlayClick()
                                isPlaying = true
                            }
                        }
                    ) {
                        if (isPlaying) {
                            Text(
                                text = "⏸",
                                color = Color.White,
                                fontSize = 32.sp
                            )
                        } else {
                            Icon(
                                imageVector = Icons.Default.PlayArrow,
                                contentDescription = "Play",
                                tint = Color.White,
                                modifier = Modifier.size(32.dp)
                            )
                        }
                    }
                    
                    IconButton(
                        onClick = {
                            onStopClick()
                            isPlaying = false
                        }
                    ) {
                        Text(
                            text = "⏹",
                            color = Color.White,
                            fontSize = 32.sp
                        )
                    }
                }
            }
        }
    }
    
    // Auto-hide controls after 3 seconds
    LaunchedEffect(showControls) {
        if (showControls) {
            kotlinx.coroutines.delay(3000)
            showControls = false
        }
    }
}
