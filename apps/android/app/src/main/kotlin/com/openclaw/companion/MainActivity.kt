package com.openclaw.companion

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                OpenClawApp()
            }
        }
    }
}

@Composable
fun OpenClawApp() {
    val navController = rememberNavController()
    var gatewayUrl by remember { mutableStateOf("") }

    NavHost(navController, startDestination = "pair") {
        composable("pair") {
            PairScreen(
                onPaired = { url ->
                    if (url == "__SCAN__") {
                        navController.navigate("scan")
                    } else {
                        gatewayUrl = url
                        navController.navigate("console") {
                            popUpTo("pair") { inclusive = true }
                        }
                    }
                }
            )
        }
        composable("console") {
            ConsoleScreen(
                gatewayUrl = gatewayUrl,
                onDisconnect = {
                    gatewayUrl = ""
                    navController.navigate("pair") {
                        popUpTo("console") { inclusive = true }
                    }
                }
            )
        }
        composable("scan") {
            ScanScreen(
                onScanned = { url ->
                    gatewayUrl = url
                    navController.navigate("console") {
                        popUpTo("pair") { inclusive = true }
                    }
                },
                onBack = { navController.popBackStack() }
            )
        }
    }
}

@Composable
fun PairScreen(onPaired: (String) -> Unit) {
    var manualUrl by remember { mutableStateOf("") }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text("OpenClaw", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            "连接到你的 OpenClaw Gateway",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
        Spacer(modifier = Modifier.height(48.dp))

        Button(
            onClick = { onPaired("__SCAN__") },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("扫码配对")
        }

        Spacer(modifier = Modifier.height(24.dp))
        Text("或手动输入地址", color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(modifier = Modifier.height(12.dp))

        OutlinedTextField(
            value = manualUrl,
            onValueChange = { manualUrl = it },
            label = { Text("Gateway 地址") },
            placeholder = { Text("http://192.168.1.x:18789") },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true
        )

        Spacer(modifier = Modifier.height(16.dp))

        OutlinedButton(
            onClick = { if (manualUrl.isNotBlank()) onPaired(manualUrl.trim()) },
            modifier = Modifier.fillMaxWidth(),
            enabled = manualUrl.isNotBlank()
        ) {
            Text("连接")
        }
    }
}
