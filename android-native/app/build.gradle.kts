plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.mikerosoft.threadbear.android"
    compileSdk = 37

    defaultConfig {
        // Mike's own, clearly apart from Keith's wrapper (com.tinkerware.threadbear).
        applicationId = "com.mikerosoft.threadbear.android"
        minSdk = 29
        targetSdk = 36
        versionCode = 1
        versionName = "0.1"
        // Phones only: keeps the APK (and this box's disk) small.
        ndk { abiFilters += "arm64-v8a" }
    }

    buildTypes {
        debug {
            buildConfigField("boolean", "ALLOW_LOCAL", "true")
        }
        release {
            isMinifyEnabled = false
            buildConfigField("boolean", "ALLOW_LOCAL", "false")
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

}

kotlin { jvmToolchain(17) }

dependencies {
    implementation(project(":kit"))
    implementation("dev.convex:android-convexmobile:0.8.0@aar") { isTransitive = true }
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    implementation(platform("androidx.compose:compose-bom:2026.09.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui-tooling-preview")
    debugImplementation("androidx.compose.ui:ui-tooling")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.11.0")
    implementation("androidx.lifecycle:lifecycle-process:2.11.0")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.11.0")
    implementation("androidx.navigation:navigation-compose:2.10.2")
    implementation("androidx.core:core-ktx:1.19.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
    implementation("com.google.android.gms:play-services-code-scanner:16.1.0")
}
