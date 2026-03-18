package com.openclaw.server

import android.content.Intent
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.*
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

class MainActivity : ComponentActivity() {

    private lateinit var nodeManager: NodeManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        nodeManager = NodeManager(applicationContext)

        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    ServerApp(nodeManager, ::startService, ::stopService)
                }
            }
        }
    }

    private fun startService() {
        val intent = Intent(this, GatewayService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun stopService() {
        val intent = Intent(this, GatewayService::class.java).apply { action = "STOP" }
        startService(intent)
    }
}

enum class Phase {
    IDLE, INSTALLING, RUNNING, ERROR
}

@Composable
fun ServerApp(
    nodeManager: NodeManager,
    onStartService: () -> Unit,
    onStopService: () -> Unit
) {
    var phase by remember { mutableStateOf(Phase.IDLE) }
    var statusText by remember { mutableStateOf("") }
    var errorText by remember { mutableStateOf<String?>(null) }
    var healthy by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    // Check initial state
    LaunchedEffect(Unit) {
        if (nodeManager.isOpenclawInstalled) {
            phase = Phase.RUNNING
        }
    }

    // Health check loop when running
    LaunchedEffect(phase) {
        if (phase == Phase.RUNNING) {
            while (true) {
                healthy = checkHealth()
                delay(5000)
            }
        }
    }

    fun install() {
        phase = Phase.INSTALLING
        errorText = null
        scope.launch(Dispatchers.IO) {
            try {
                // Step 1: Extract Node.js
                nodeManager.extractNode { msg ->
                    statusText = msg
                }.getOrThrow()

                // Step 2: Install OpenClaw
                nodeManager.installOpenclaw { msg ->
                    statusText = msg
                }.getOrThrow()

                // Step 3: Start service
                statusText = "启动 Gateway..."
                withContext(Dispatchers.Main) { onStartService() }

                // Step 4: Wait for healthy
                statusText = "等待服务就绪..."
                var attempts = 0
                while (attempts < 30) {
                    delay(2000)
                    if (checkHealth()) break
                    attempts++
                }

                phase = Phase.RUNNING
                statusText = ""
            } catch (e: Exception) {
                errorText = e.message ?: "未知错误"
                phase = Phase.ERROR
            }
        }
    }

    fun uninstall() {
        scope.launch(Dispatchers.IO) {
            withContext(Dispatchers.Main) { onStopService() }
            delay(2000)
            nodeManager.uninstallOpenclaw()
            nodeManager.removeAll()
            phase = Phase.IDLE
            statusText = ""
            errorText = null
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text("OpenClaw Server", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            "在手机上直接运行 AI Gateway",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(modifier = Modifier.height(40.dp))

        when (phase) {
            Phase.IDLE -> {
                Button(
                    onClick = { install() },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(56.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = Color(0xFF6C5CE7)
                    )
                ) {
                    Text("一键安装", fontSize = 18.sp)
                }
                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    "将自动安装 Node.js 运行时与 OpenClaw 服务",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            Phase.INSTALLING -> {
                CircularProgressIndicator(color = Color(0xFF6C5CE7))
                Spacer(modifier = Modifier.height(16.dp))
                Text(statusText, style = MaterialTheme.typography.bodyMedium)
            }

            Phase.RUNNING -> {
                // Status card
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant
                    )
                ) {
                    Column(modifier = Modifier.padding(20.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text(
                                if (healthy) "●" else "●",
                                color = if (healthy) Color(0xFF2ECC71) else Color(0xFFE74C3C),
                                fontSize = 16.sp
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("Gateway 状态")
                            Spacer(modifier = Modifier.weight(1f))
                            Text(
                                if (healthy) "运行中" else "未响应",
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Spacer(modifier = Modifier.height(12.dp))
                        HorizontalDivider()
                        Spacer(modifier = Modifier.height(12.dp))
                        Row {
                            Text("地址")
                            Spacer(modifier = Modifier.weight(1f))
                            Text(
                                "localhost:18789",
                                fontFamily = FontFamily.Monospace,
                                fontSize = 13.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }

                Spacer(modifier = Modifier.height(24.dp))

                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    if (!healthy) {
                        OutlinedButton(
                            onClick = { onStartService() },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("启动")
                        }
                    } else {
                        OutlinedButton(
                            onClick = { onStopService() },
                            modifier = Modifier.weight(1f)
                        ) {
                            Text("停止")
                        }
                    }
                    Button(
                        onClick = { uninstall() },
                        modifier = Modifier.weight(1f),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color(0xFFE74C3C)
                        )
                    ) {
                        Text("卸载")
                    }
                }
            }

            Phase.ERROR -> {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = Color(0x1AE74C3C)
                    )
                ) {
                    Text(
                        errorText ?: "未知错误",
                        modifier = Modifier.padding(16.dp),
                        color = Color(0xFFE74C3C),
                        fontSize = 13.sp
                    )
                }
                Spacer(modifier = Modifier.height(16.dp))
                Button(
                    onClick = { install() },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("重试")
                }
            }
        }
    }
}

private val httpClient = OkHttpClient.Builder()
    .connectTimeout(3, TimeUnit.SECONDS)
    .readTimeout(3, TimeUnit.SECONDS)
    .build()

private suspend fun checkHealth(): Boolean = withContext(Dispatchers.IO) {
    try {
        val request = Request.Builder().url("http://localhost:18789").build()
        val response = httpClient.newCall(request).execute()
        response.isSuccessful
    } catch (_: Exception) {
        false
    }
}
