package com.proximyst.birbfetcher;

public class Constants {
  private Constants() throws IllegalAccessException {
    throw new IllegalAccessException("com.proximyst.birbfetcher.Constants cannot be instantiated.");
  }

  public static final int DUPLICATE_KEY_NAME_ERROR_CODE = 1061;
  public static final int DUPLICATE_VALUE_ERROR_CODE = 1062;
}