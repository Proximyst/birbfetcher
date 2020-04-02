package com.proximyst.birbfetcher.utils;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;

public class Hashing {
  private Hashing() throws IllegalAccessException {
    throw new IllegalAccessException("com.proximyst.birbfetcher.utils.Hashing cannot be instantiated.");
  }

  public static byte[] hash(String algorithm, byte[] data) throws NoSuchAlgorithmException {
    return MessageDigest.getInstance(algorithm).digest(data);
  }

  public static byte[] sha256(byte[] data) {
    try {
      return hash("SHA-256", data);
    } catch (NoSuchAlgorithmException ex) {
      // Should not be reachable.
      throw new RuntimeException(ex);
    }
  }
}