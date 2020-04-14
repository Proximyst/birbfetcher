package com.proximyst.birbfetcher.utils;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Collection;
import java.util.Objects;
import java.util.StringJoiner;
import java.util.function.Function;
import java.util.stream.Stream;
import org.jetbrains.annotations.NotNull;

public class Functions {
  private Functions() throws IllegalAccessException {
    throw new IllegalAccessException("com.proximyst.birbfetcher.utils.Functions cannot be instantiated.");
  }

  private static final char[] HEXADECIMAL_CHARACTERS = "0123456789ABCDEF".toCharArray();

  public static <T> StringJoiner addCollection(
      Collection<T> collection,
      Function<T, String> toString,
      StringJoiner joiner
  ) {
    for (T t : collection) {
      joiner.add(toString.apply(t));
    }
    return joiner;
  }

  public static <T> StringJoiner addCollection(Collection<T> collection, StringJoiner joiner) {
    return addCollection(collection, Objects::toString, joiner);
  }

  public static boolean endsWithAny(String string, String... endings) {
    return endsWithAny(string, false, endings);
  }

  public static boolean endsWithAny(String string, boolean ignoreCase, String... endings) {
    if (!ignoreCase) {
      return Stream.of(endings).anyMatch(string::endsWith);
    } else {
      String checked = string.toLowerCase();
      return Stream.of(endings).map(String::toLowerCase).anyMatch(checked::endsWith);
    }
  }

  public static byte[] readEntireStream(InputStream stream)
      throws IOException {
    return readEntireStream(stream, Short.MAX_VALUE);
  }

  public static byte[] readEntireStream(InputStream stream, int bufferSize)
      throws IOException {
    var read = 0;
    var buf = new byte[bufferSize];
    var byteStream = new ByteArrayOutputStream(bufferSize);

    while ((read = stream.read(buf)) > 0) {
      byteStream.write(buf, 0, read);
    }

    return byteStream.toByteArray();
  }

  public static byte[] readByteArray(InputStream stream, int maxBytes)
      throws IOException {
    return readByteArray(stream, maxBytes, Short.MAX_VALUE);
  }

  public static byte[] readByteArray(InputStream stream, int maxBytes, int bufferSize)
      throws IOException {
    var read = 0;
    var buf = new byte[bufferSize];
    var byteStream = new ByteArrayOutputStream(bufferSize);

    while ((read = stream.read(buf)) > 0) {
      byteStream.write(buf, 0, read);
      if (maxBytes > 0 && byteStream.size() > maxBytes) {
        return null;
      }
    }

    return byteStream.toByteArray();
  }

  @NotNull
  public static String bytesToHex(@NotNull byte[] bytes) {
    char[] hexChars = new char[bytes.length * 2];

    for (int j = 0; j < bytes.length; j++) {
      int character = bytes[j] & 0xFF;
      hexChars[j * 2] = HEXADECIMAL_CHARACTERS[character >>> 4];
      hexChars[j * 2 + 1] = HEXADECIMAL_CHARACTERS[character & 0x0F];
    }

    return new String(hexChars);
  }
}