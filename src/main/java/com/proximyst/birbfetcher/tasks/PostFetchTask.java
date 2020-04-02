package com.proximyst.birbfetcher.tasks;

import com.proximyst.birbfetcher.Main;
import com.proximyst.birbfetcher.api.RedditPost.JsonResponse;
import com.proximyst.birbfetcher.api.RedditPostType;
import java.util.Arrays;
import java.util.TimerTask;
import org.jetbrains.annotations.NotNull;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class PostFetchTask extends TimerTask {
  private static final Logger logger = LoggerFactory.getLogger(PostFetchTask.class);

  private final Main main;

  public PostFetchTask(Main main) {
    this.main = main;
  }

  @Override
  public void run() {
    main.getConfiguration().getSubreddits().parallelStream()
        .forEach(sub -> Arrays.stream(RedditPostType.values()).forEach(type -> fetchSub(sub, type)));
  }

  private void fetchSub(String subreddit, RedditPostType postType) {
    main.getRedditApi().getPosts(subreddit, postType)
        .enqueue(new Callback<>() {
      @Override
      public void onResponse(
          @NotNull Call<JsonResponse> call,
          @NotNull Response<JsonResponse> response
      ) {
        if (!response.isSuccessful()) {
          logger.warn(
              "Got status code "
                  + response.code()
                  + " for subreddit "
                  + subreddit
                  + " of type "
                  + postType
                  + "!"
          );
          return;
        }

        // Body is only null if #isSuccessful is false.
        assert response.body() != null;

        logger.info("Got response for subreddit " + subreddit + " of type " + postType + ". "
            + "Received " + response.body().getPosts().size() + " posts.");
        response.body().getPosts().forEach(main.getPostQueue()::offer);
      }

      @Override
      public void onFailure(@NotNull Call<JsonResponse> call, @NotNull Throwable throwable) {
        logger.error("Error upon call to " + call.request().url() + "!", throwable);
      }
    });
  }
}
