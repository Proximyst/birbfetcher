package com.proximyst.birbfetcher.http;

import com.proximyst.birbfetcher.Main;
import com.proximyst.birbfetcher.utils.Functions;
import java.sql.SQLException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import spark.Request;
import spark.Response;
import spark.Route;

public class RandomImage implements Route {
  private static final Logger logger = LoggerFactory.getLogger(RandomImage.class);

  private final Main main;

  public RandomImage(Main main) {
    this.main = main;
  }

  @Override
  public Object handle(Request request, Response response) throws Exception {
    try (var connection = main.getHikariDataSource().getConnection();
        var randomImage = connection.createStatement();
        var result = randomImage.executeQuery(
            // ORDER BY RAND() is not slow till we reach sizes to worry about in a year's time, if
            // even that soon.
            "SELECT id, image, content_type, permalink FROM birbs WHERE banned=false ORDER BY RAND() LIMIT 1"
        )) {
      if (!result.next()) {
        throw new IllegalStateException("no images are available");
      }

      var id = result.getInt("id");
      var image = result.getBlob("image");
      var contentType = result.getString("content_type");
      var permalink = result.getString("permalink");

      response.status(200);
      response.type(contentType);
      response.cookie("Permalink", permalink, -1);
      response.cookie("Id", String.valueOf(id), -1);
      return Functions.readEntireStream(image.getBinaryStream());
    } catch (SQLException ex) {
      logger.error("Something went wrong when fetching an image.", ex);
      response.status(500);
      return "Something went wrong.";
    }
  }
}
