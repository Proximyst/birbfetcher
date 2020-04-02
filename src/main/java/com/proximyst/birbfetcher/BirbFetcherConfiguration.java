package com.proximyst.birbfetcher;

import java.util.Set;
import org.jetbrains.annotations.NotNull;

public class BirbFetcherConfiguration {
  @NotNull
  private final String apiBaseUrl;

  @NotNull
  private final String userAgent;

  @NotNull
  private final String jdbc;

  @NotNull
  private final String sqlUsername;

  @NotNull
  private final String sqlPassword;

  @NotNull
  private final Set<String> subreddits;

  private final int threadPoolSize;

  public BirbFetcherConfiguration(
      @NotNull String apiBaseUrl,
      @NotNull String userAgent,
      @NotNull String jdbc,
      @NotNull String sqlUsername,
      @NotNull String sqlPassword,
      @NotNull Set<String> subreddits,
      int threadPoolSize
  ) {
    this.apiBaseUrl = apiBaseUrl;
    this.userAgent = userAgent;
    this.jdbc = jdbc;
    this.sqlUsername = sqlUsername;
    this.sqlPassword = sqlPassword;
    this.subreddits = subreddits;
    this.threadPoolSize = threadPoolSize;
  }

  @NotNull
  public String getApiBaseUrl() {
    return apiBaseUrl;
  }

  @NotNull
  public String getUserAgent() {
    return userAgent;
  }

  @NotNull
  public String getJdbc() {
    return jdbc;
  }

  @NotNull
  public String getSqlUsername() {
    return sqlUsername;
  }

  @NotNull
  public String getSqlPassword() {
    return sqlPassword;
  }

  @NotNull
  public Set<String> getSubreddits() {
    return subreddits;
  }

  public int getThreadPoolSize() {
    return threadPoolSize;
  }
}
