plugins {
    java
    application
    id("com.github.johnrengelman.shadow") version "5.2.0"
}

group = "com.proximyst"
version = "0.1.0"

repositories {
    jcenter()
    mavenCentral()
}

dependencies {
    implementation("com.squareup.retrofit2:retrofit:2.8.+")
    implementation("com.squareup.retrofit2:converter-gson:2.8.+")
    implementation("com.google.code.gson:gson:2.8.+")
    implementation("org.itishka.gson-flatten:gson-flatten:0.8.+")

    implementation("org.mariadb.jdbc:mariadb-java-client:2.6.+")
    implementation("com.zaxxer:HikariCP:3.4.+")
    implementation("com.moandjiezana.toml:toml4j:0.7.+")

    implementation("org.slf4j:slf4j-api:1.7.+")
    implementation("ch.qos.logback:logback-core:1.2.+")
    implementation("ch.qos.logback:logback-classic:1.2.+")

    implementation("com.sparkjava:spark-core:2.8.+")

    implementation("org.jetbrains:annotations:19.0.0")
}

configure<JavaPluginConvention> {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = sourceCompatibility
}

application{
    mainClassName = "com.proximyst.birbfetcher.Main"
}