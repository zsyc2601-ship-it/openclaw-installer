package com.openclaw.server

import android.content.Context
import android.util.Log
import java.io.File
import java.io.FileOutputStream

/**
 * Manages the embedded Node.js runtime on Android.
 *
 * Node.js for Android is built against Bionic libc (not glibc).
 * We use the Termux-compatible prebuilt binary placed in assets/node-android-arm64.
 *
 * Directory layout after setup:
 *   {filesDir}/node/bin/node       — Node.js binary
 *   {filesDir}/node/bin/npm        — npm symlink
 *   {filesDir}/node/lib/...        — Node.js stdlib
 *   {filesDir}/node/bin/openclaw   — installed via npm
 */
class NodeManager(private val context: Context) {

    companion object {
        private const val TAG = "NodeManager"
        private const val NODE_ASSET = "node-arm64.tar.xz"
        private const val MARKER_FILE = ".node-extracted"
    }

    val baseDir: File get() = File(context.filesDir, "node")
    val nodeBin: File get() = File(baseDir, "bin/node")
    val npmBin: File get() = File(baseDir, "lib/node_modules/npm/bin/npm-cli.js")
    val openclawBin: File get() = File(baseDir, "bin/openclaw")
    val logsDir: File get() = File(context.filesDir, "logs")

    val isNodeInstalled: Boolean get() = nodeBin.exists() && nodeBin.canExecute()
    val isOpenclawInstalled: Boolean get() = openclawBin.exists()

    private val env: Map<String, String>
        get() {
            val binDir = File(baseDir, "bin").absolutePath
            val libDir = File(baseDir, "lib").absolutePath
            val systemPath = System.getenv("PATH") ?: "/system/bin"
            return mapOf(
                "PATH" to "$binDir:$systemPath",
                "HOME" to context.filesDir.absolutePath,
                "TMPDIR" to context.cacheDir.absolutePath,
                "NODE_PATH" to "$libDir/node_modules",
                "npm_config_prefix" to baseDir.absolutePath,
                "npm_config_cache" to File(context.cacheDir, "npm-cache").absolutePath,
            )
        }

    /**
     * Extract Node.js from assets if not already done.
     */
    fun extractNode(onProgress: (String) -> Unit): Result<Unit> = runCatching {
        val marker = File(baseDir, MARKER_FILE)
        if (marker.exists() && isNodeInstalled) {
            onProgress("Node.js 已就绪")
            return Result.success(Unit)
        }

        onProgress("正在释放 Node.js 运行时...")

        // Clean up partial extraction
        if (baseDir.exists()) baseDir.deleteRecursively()
        baseDir.mkdirs()

        // Extract tar.xz from assets
        val assetStream = context.assets.open(NODE_ASSET)
        val tmpFile = File(context.cacheDir, NODE_ASSET)
        FileOutputStream(tmpFile).use { out -> assetStream.copyTo(out) }
        assetStream.close()

        // Use system tar if available, else use ProcessBuilder
        onProgress("正在解压 Node.js...")
        val proc = ProcessBuilder("tar", "xJf", tmpFile.absolutePath, "--strip-components=1", "-C", baseDir.absolutePath)
            .redirectErrorStream(true)
            .start()
        val output = proc.inputStream.bufferedReader().readText()
        val exitCode = proc.waitFor()

        tmpFile.delete()

        if (exitCode != 0) {
            throw RuntimeException("tar extract failed ($exitCode): $output")
        }

        // Make node executable
        nodeBin.setExecutable(true)

        // Write marker
        marker.writeText("ok")
        onProgress("Node.js 释放完成")

        Log.i(TAG, "Node.js extracted to ${baseDir.absolutePath}")
    }

    /**
     * Install openclaw globally via npm.
     */
    fun installOpenclaw(onProgress: (String) -> Unit): Result<Unit> = runCatching {
        if (!isNodeInstalled) throw RuntimeException("Node.js not installed")

        onProgress("正在安装 OpenClaw...")

        // Detect npm registry
        val registry = pickRegistry()
        Log.i(TAG, "Using npm registry: $registry")

        val proc = ProcessBuilder(
            nodeBin.absolutePath,
            npmBin.absolutePath,
            "install", "-g", "openclaw@latest",
            "--prefix", baseDir.absolutePath,
            "--registry", registry
        )
            .directory(baseDir)
            .redirectErrorStream(true)
            .apply { environment().putAll(env) }
            .start()

        // Stream output
        proc.inputStream.bufferedReader().forEachLine { line ->
            Log.d(TAG, "npm: $line")
            if (line.contains("added") || line.contains("openclaw")) {
                onProgress("npm: $line")
            }
        }

        val exitCode = proc.waitFor()
        if (exitCode != 0) {
            throw RuntimeException("npm install failed (exit $exitCode)")
        }

        if (!isOpenclawInstalled) {
            throw RuntimeException("openclaw binary not found after install")
        }

        onProgress("OpenClaw 安装完成")
        Log.i(TAG, "openclaw installed at ${openclawBin.absolutePath}")
    }

    /**
     * Uninstall openclaw.
     */
    fun uninstallOpenclaw(): Result<Unit> = runCatching {
        if (!isNodeInstalled) return Result.success(Unit)

        val proc = ProcessBuilder(
            nodeBin.absolutePath, npmBin.absolutePath,
            "uninstall", "-g", "openclaw",
            "--prefix", baseDir.absolutePath
        )
            .directory(baseDir)
            .redirectErrorStream(true)
            .apply { environment().putAll(env) }
            .start()
        proc.waitFor()
    }

    /**
     * Start the openclaw gateway process. Returns the Process handle.
     */
    fun startGateway(): Process {
        logsDir.mkdirs()
        val outLog = File(logsDir, "gateway.out.log")
        val errLog = File(logsDir, "gateway.err.log")

        return ProcessBuilder(nodeBin.absolutePath, openclawBin.absolutePath, "up")
            .directory(context.filesDir)
            .redirectOutput(outLog)
            .redirectError(errLog)
            .apply { environment().putAll(env) }
            .start()
    }

    /**
     * Remove everything — Node.js + openclaw + logs.
     */
    fun removeAll() {
        if (baseDir.exists()) baseDir.deleteRecursively()
        if (logsDir.exists()) logsDir.deleteRecursively()
        // Remove openclaw config
        val configDir = File(context.filesDir, ".openclaw")
        if (configDir.exists()) configDir.deleteRecursively()
    }

    private fun pickRegistry(): String {
        return try {
            val proc = ProcessBuilder("ping", "-c", "1", "-W", "2", "registry.npmmirror.com")
                .redirectErrorStream(true)
                .start()
            val ok = proc.waitFor() == 0
            if (ok) "https://registry.npmmirror.com" else "https://registry.npmjs.org"
        } catch (_: Exception) {
            "https://registry.npmjs.org"
        }
    }
}
