package com.proximyst.birbfetcher.tasks;

import com.proximyst.birbfetcher.Constants;
import com.proximyst.birbfetcher.Main;
import com.proximyst.birbfetcher.api.RedditPost;
import com.proximyst.birbfetcher.utils.Functions;
import com.proximyst.birbfetcher.utils.Hashing;
import java.io.BufferedOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.sql.Blob;
import java.sql.SQLException;
import java.util.TimerTask;
import javax.sql.rowset.serial.SerialBlob;
import okhttp3.Call;
import okhttp3.Callback;
import okhttp3.Request;
import okhttp3.Response;
import org.jetbrains.annotations.NotNull;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class PostProcessTask extends TimerTask {
  private static final Logger logger = LoggerFactory.getLogger(PostProcessTask.class);

  private final Main main;

  public PostProcessTask(Main main) {
    this.main = main;
  }

  @Override
  public void run() {
    var processed = 0;
    var queued = 0;

    RedditPost post;
    while ((post = main.getPostQueue().poll()) != null) {
      ++processed;
      if (!post.isSafe()
          || post.getPostHint() == null
          || !post.getPostHint().equals("image")) {
        continue;
      }

      // Effectively final for the lambda.
      final var finalPost = post;

      main.getExecutor().submit(() -> handlePostSource(finalPost, finalPost.getUrl()));
      ++queued;
    }
    logger.info("Processed " + processed + " posts, queued " + queued + " handlers.");
  }

  private void handlePostSource(RedditPost post, String source) {
    main.getHttpClient().newCall(
        new Request.Builder()
            .url(source)
            .get()
            .build()
    ).enqueue(new Callback() {
      @Override
      public void onFailure(@NotNull Call call, @NotNull IOException ex) {
        logger.error("Could not request the image of post " + post.getPermalink(), ex);
      }

      @Override
      public void onResponse(@NotNull Call call, @NotNull Response response) throws IOException {
        if (!response.isSuccessful()) {
          logger.warn("Post " + post.getPermalink() + " could not be fetched. HTTP status: " + response.code());
          return;
        }

        try (var body = response.body()) {
          // Only null when #isSuccessful is false.
          assert body != null;

          var contentType = body.contentType();
          if (contentType == null) {
            logger.info("No content type found for " + post.getPermalink());
            return;
          }

          if (body.contentLength() >= 1024 * 1024 * 16) {
            logger.info("Post is over 16MB (content length): " + post.getPermalink());
            return;
          }

          var bodyBytes = Functions.readByteArray(body.byteStream(), 1024 * 1024 * 16);
          if (bodyBytes == null) {
            // The body was too large.
            logger.info("Post is over 16MB (readByteArray): " + post.getPermalink());
            return;
          }

          main.getExecutor().submit(() -> savePost(post, source, contentType.toString(), bodyBytes));
        }
      }
    });
  }

  private void savePost(RedditPost post, String source, String contentType, byte[] body) {
    var hash = Hashing.sha256(body);
    Blob hashBlob;
    try {
      hashBlob = new SerialBlob(hash);
    } catch (SQLException ex) {
      logger.error("Could not make blob of hash.", ex);
      return;
    }
    var permalink = post.getPermalink();

    var file = new File(Main.getBirbDirectory(), Functions.bytesToHex(hash));
    if (file.isFile()) {
      // Already exists.
      return;
    }

    try {
      file.createNewFile();
      try (var stream = new FileOutputStream(file);
          var buffered = new BufferedOutputStream(stream)) {
        buffered.write(body);
      }
    } catch (IOException ex) {
      logger.error("Could not save image of hash " + Functions.bytesToHex(hash), ex);
      return;
    }

    try (var connection = main.getHikariDataSource().getConnection();
        var insert = connection.prepareStatement(
            "INSERT INTO birbs (hash, permalink, source_url, content_type) VALUES (?, ?, ?, ?)"
        )) {
      insert.setBlob(1, hashBlob);
      insert.setString(2, permalink);
      insert.setString(3, source);
      insert.setString(4, contentType);
      insert.executeUpdate();
    } catch (SQLException ex) {
      if (ex.getErrorCode() != Constants.DUPLICATE_VALUE_ERROR_CODE) {
        logger.error("Could not insert image of hash " + Functions.bytesToHex(hash) + " at " + permalink, ex);
      }
    }
  }
}
