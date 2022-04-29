package com.brave.adblock

import android.annotation.SuppressLint
import android.graphics.Bitmap
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.view.View
import android.webkit.*
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import com.brave.adblock.databinding.ActivityMainBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.io.File

class MainActivity : AppCompatActivity() {
    companion object {
        val TAG: String = MainActivity::class.java.simpleName
    }

    private lateinit var vb: ActivityMainBinding
    private val client = AdBlockClient()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        vb = ActivityMainBinding.inflate(layoutInflater)
        setContentView(vb.root)

        configureWebView()

        lifecycleScope.launch {
            launch(Dispatchers.IO) ioLaunch@ {
                val serializedFile = File(filesDir, "serialized.dat")
                if (serializedFile.exists() && client.deserialize(serializedFile.absolutePath)) {
                    return@ioLaunch
                }
                val easyList = assets.open("easylist.txt").bufferedReader().use { it.readText() }
                client.loadRules(easyList)
                client.serialize(serializedFile.absolutePath)
            }.join()
            vb.flProgress.visibility = View.GONE
        }

        vb.btnGo.setOnClickListener {
            val url = vb.etUrl.text.toString()
            vb.webView.loadUrl(url)
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configureWebView() {
        with(vb.webView.settings) {
            javaScriptEnabled = true
            databaseEnabled = true
            useWideViewPort = true
            domStorageEnabled = true
            mediaPlaybackRequiresUserGesture = false
        }
        vb.webView.webViewClient = object : WebViewClient() {
            private var baseUri: Uri? = null
            @Volatile
            private var blockedAds = 0
            override fun shouldInterceptRequest(view: WebView?, request: WebResourceRequest): WebResourceResponse? {
                return if (baseUri != null && client.matches(request.url.toString(), Utils.mapRequestToFilterOption(request), baseUri.toString())) {
                    Log.w(TAG, "Blocked ads request: ${request.url}")
                    blockedAds++
                    vb.webView.handler.post {
                        vb.tvBlockedAds.text = getString(R.string.ads_blocked_fmt, blockedAds)
                    }
                    WebResourceResponse("text/plain", "utf-8", "".byteInputStream())
                } else {
                    Log.i(TAG, "Request allowed: ${request.url}")
                    super.shouldInterceptRequest(view, request)
                }
            }

            override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
                blockedAds = 0
                baseUri = Uri.parse(url)
                super.onPageStarted(view, url, favicon)
            }
        }
    }
}