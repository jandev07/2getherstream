plugins { id("com.android.application"); id("org.jetbrains.kotlin.android"); id("org.jetbrains.kotlin.plugin.compose") }
android {
    namespace = "app.together.v2"
    compileSdk = 36
    defaultConfig { applicationId = "app.together.v2"; minSdk = 26; targetSdk = 36; versionCode = 9; versionName = "0.1.8-private-alpha"; testInstrumentationRunner = "app.together.v2.MutedTestRunner"; ndk { abiFilters += listOf("arm64-v8a", "x86_64") } }
    buildFeatures { compose = true; buildConfig = true }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    // libmpvKt packages the native engine for both proof/device ABIs.
}
kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }
dependencies {
    implementation(platform("androidx.compose:compose-bom:2025.04.01"))
    implementation("androidx.activity:activity-compose:1.10.1")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.material3:material3")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("io.github.yuroyami:libmpvkt-view:0.3.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.11.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test:runner:1.7.0")
    androidTestImplementation("androidx.test:core:1.7.0")
    androidTestImplementation("androidx.test.ext:junit:1.3.0")
}
