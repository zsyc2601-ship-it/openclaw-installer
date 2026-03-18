package com.openclaw.server

import android.app.*
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import kotlinx.coroutines.*

/**
 * Foreground service that keeps the OpenClaw Gateway process alive.
 */
class GatewayService : Service() {

    companion object {
        private const val TAG = "GatewayService"
        private const val CHANNEL_ID = "openclaw_gateway"
        private const val NOTIFICATION_ID = 1
    }

    private var gatewayProcess: Process? = null
    private var wakeLock: PowerManager.WakeLock? = null
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val action = intent?.action

        when (action) {
            "STOP" -> {
                stopGateway()
                stopForeground(STOP_FOREGROUND_REMOVE)
                stopSelf()
                return START_NOT_STICKY
            }
        }

        // Start foreground
        val notification = buildNotification("OpenClaw Gateway 运行中")
        startForeground(NOTIFICATION_ID, notification)

        // Acquire wake lock to prevent CPU sleep
        val pm = getSystemService(POWER_SERVICE) as PowerManager
        wakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "openclaw:gateway")
        wakeLock?.acquire(24 * 60 * 60 * 1000L) // 24h max

        // Start gateway process
        startGateway()

        return START_STICKY
    }

    private fun startGateway() {
        scope.launch {
            try {
                val nodeManager = NodeManager(applicationContext)
                if (!nodeManager.isOpenclawInstalled) {
                    Log.e(TAG, "OpenClaw not installed")
                    return@launch
                }

                val process = nodeManager.startGateway()
                gatewayProcess = process

                Log.i(TAG, "Gateway process started (pid ${process.pid()})")

                // Monitor process — restart if crashed
                val exitCode = process.waitFor()
                Log.w(TAG, "Gateway exited with code $exitCode, restarting in 5s...")

                delay(5000)
                if (gatewayProcess != null) {
                    startGateway() // auto-restart
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start gateway", e)
            }
        }
    }

    private fun stopGateway() {
        gatewayProcess?.destroyForcibly()
        gatewayProcess = null
        scope.cancel()
    }

    override fun onDestroy() {
        stopGateway()
        wakeLock?.let { if (it.isHeld) it.release() }
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "OpenClaw Gateway",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "OpenClaw Gateway 后台运行状态"
            }
            val nm = getSystemService(NotificationManager::class.java)
            nm.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(text: String): Notification {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val stopIntent = Intent(this, GatewayService::class.java).apply { action = "STOP" }
        val stopPendingIntent = PendingIntent.getService(
            this, 1, stopIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        return Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("OpenClaw Gateway")
            .setContentText(text)
            .setSmallIcon(android.R.drawable.ic_menu_manage)
            .setContentIntent(pendingIntent)
            .addAction(Notification.Action.Builder(
                null, "停止", stopPendingIntent
            ).build())
            .setOngoing(true)
            .build()
    }

    private fun Process.pid(): Long {
        return try { pid().toLong() } catch (_: Exception) { -1 }
    }
}
