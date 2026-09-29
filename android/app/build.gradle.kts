plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "com.mikey"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.mikey"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
    }

    // libopus and the JNI wrapper. Needs CMake and the submodule (see docs/DEVELOPER_PLAYBOOKS_AND_SKILLS.md).
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
            // Any CMake from here up, whether from the SDK manager or the PATH.
            version = "3.22.1+"
        }
    }
}

dependencies {
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.foundation)
    implementation(libs.androidx.camera.core)
    implementation(libs.androidx.camera.camera2)
    implementation(libs.androidx.camera.lifecycle)

    testImplementation(libs.junit)
}
