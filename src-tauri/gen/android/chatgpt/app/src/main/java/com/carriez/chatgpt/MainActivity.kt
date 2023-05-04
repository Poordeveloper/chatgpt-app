package com.carriez.chatgpt

import app.tauri.plugin.PluginManager

class MainActivity : TauriActivity() {
  var pluginManager: PluginManager = PluginManager(this)
}
