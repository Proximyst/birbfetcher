package com.proximyst.birbfetcher.api;

public enum RedditPostType {
  NEW,
  HOT,
  ;

  @Override
  public String toString() {
    return name().toLowerCase();
  }
}
