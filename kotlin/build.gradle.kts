plugins {
    kotlin("jvm") version "2.1.20"
    application
}

repositories { mavenCentral() }

dependencies {
    implementation("org.json:json:20240303")
}

application {
    mainClass.set("nxs.TestKt")
}

tasks.register<JavaExec>("bench") {
    group = "application"
    classpath = sourceSets["main"].runtimeClasspath
    mainClass.set("nxs.BenchKt")
    args = listOf("../js/fixtures")
}

kotlin {
    jvmToolchain(25)
}

// No Java sources in this project — suppress the toolchain consistency check.
tasks.withType<JavaCompile> {
    options.release.set(23)
}
