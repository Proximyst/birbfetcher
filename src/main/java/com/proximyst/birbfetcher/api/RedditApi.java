package com.proximyst.birbfetcher.api;

import retrofit2.Call;
import retrofit2.http.GET;
import retrofit2.http.Path;

public interface RedditApi {
  @GET("r/{subreddit}/{type}.json?limit=100")
  Call<RedditPost.JsonResponse> getPosts(
      @Path("subreddit") String subreddit,
      @Path("type") RedditPostType type
  );
}
