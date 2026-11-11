package com.mikerosoft.threadbear.kit

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * Whether there's a newer Android app than this one (docs/android-native.md,
 * "Releases"): each release/YYYY-MM-DD tag's GitHub release carries
 * threadbear-android.apk, and GitHub's "latest release" is the newest. A
 * build that isn't a release (no date) never offers one, so a debug build
 * on Mike's Pixel doesn't nag him to replace itself.
 */
object AppUpdate {
    const val ASSET = "threadbear-android.apk"

    data class Available(val date: String, val apkUrl: String, val pageUrl: String)

    private val DATE = Regex("^\\d{4}-\\d{2}-\\d{2}$")

    /** GitHub's answer for releases/latest, as text: the newer release, or null. */
    fun newer(latestJson: String, installedDate: String?): Available? {
        val mine = installedDate?.takeIf { DATE.matches(it) } ?: return null
        val o = try { Json.parseToJsonElement(latestJson) as? JsonObject } catch (_: Exception) { null } ?: return null
        val tag = o["tag_name"]?.jsonPrimitive?.takeIf { it.isString }?.content ?: return null
        val date = tag.removePrefix("release/").takeIf { tag.startsWith("release/") && DATE.matches(it) } ?: return null
        // Dates compare as text: yyyy-MM-dd sorts in time order.
        if (date <= mine) return null
        val asset = (o["assets"] as? JsonArray)?.mapNotNull { it as? JsonObject }
            ?.firstOrNull { it["name"]?.jsonPrimitive?.content == ASSET } ?: return null
        val apk = asset["browser_download_url"]?.jsonPrimitive?.content?.takeIf { it.startsWith("https://github.com/") } ?: return null
        val page = o["html_url"]?.jsonPrimitive?.content?.takeIf { it.startsWith("https://github.com/") } ?: apk
        return Available(date, apk, page)
    }

    /** Where to ask: GitHub's API for this repo's latest release. */
    fun latestUrl(repo: String): String? =
        repo.takeIf { Regex("^[A-Za-z0-9_.-]{1,100}/[A-Za-z0-9_.-]{1,100}$").matches(it) }?.let { "https://api.github.com/repos/$it/releases/latest" }
}
