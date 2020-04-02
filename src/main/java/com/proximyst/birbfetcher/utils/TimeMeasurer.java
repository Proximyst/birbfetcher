package com.proximyst.birbfetcher.utils;

public class TimeMeasurer {
  private long start = 0;

  public void start() {
    if (start != 0) {
      throw new IllegalStateException("Time measurer is already measuring time.");
    }

    start = System.currentTimeMillis();
  }

  public long stop() {
    if (start == 0) {
      throw new IllegalStateException("Time measurer is not measuring time.");
    }

    var time = System.currentTimeMillis() - start;
    start = 0;

    return time;
  }
}
