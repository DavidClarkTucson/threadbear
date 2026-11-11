package com.mikerosoft.threadbear.kit

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class AppUpdateTest {
    private fun latest(tag: String, assets: String = """[{"name":"threadbear-android.apk","browser_download_url":"https://github.com/mikecann/threadbear/releases/download/$tag/threadbear-android.apk"}]""") =
        """{"tag_name":"$tag","html_url":"https://github.com/mikecann/threadbear/releases/tag/$tag","assets":$assets}"""

    @Test fun aNewerReleaseIsOffered() {
        val a = AppUpdate.newer(latest("release/2026-11-12"), "2026-11-11")!!
        assertEquals("2026-11-12", a.date)
        assertEquals("https://github.com/mikecann/threadbear/releases/download/release/2026-11-12/threadbear-android.apk", a.apkUrl)
    }

    @Test fun theSameOrAnOlderOneIsNot() {
        assertNull(AppUpdate.newer(latest("release/2026-11-11"), "2026-11-11"))
        assertNull(AppUpdate.newer(latest("release/2026-11-10"), "2026-11-11"))
    }

    @Test fun aBuildThatIsntAReleaseNeverOffersOne() {
        assertNull(AppUpdate.newer(latest("release/2026-11-12"), ""))
        assertNull(AppUpdate.newer(latest("release/2026-11-12"), null))
    }

    @Test fun aReleaseWithoutTheApkYetIsNotOffered() {
        // The Mac's daily tag can come before the workflow has attached the APK.
        assertNull(AppUpdate.newer(latest("release/2026-11-12", "[]"), "2026-11-11"))
    }

    @Test fun oddAnswersAreNothing() {
        assertNull(AppUpdate.newer("not json", "2026-11-11"))
        assertNull(AppUpdate.newer(latest("v1.2"), "2026-11-11"))
        assertNull(AppUpdate.newer(latest("release/2026-11-12", """[{"name":"threadbear-android.apk","browser_download_url":"https://evil.example/x.apk"}]"""), "2026-11-11"))
    }

    @Test fun theUrlIsOnlyForARepoName() {
        assertEquals("https://api.github.com/repos/mikecann/threadbear/releases/latest", AppUpdate.latestUrl("mikecann/threadbear"))
        assertNull(AppUpdate.latestUrl("../../evil"))
    }
}
