package com.proximyst.birbfetcher.api;

import com.proximyst.birbfetcher.utils.Functions;
import java.util.List;
import java.util.Objects;
import java.util.StringJoiner;
import org.itishka.gsonflatten.Flatten;

public class RedditPost {
  @Flatten("data::banned_by")
  private final String bannedBy;

  @Flatten("data::subreddit")
  private final String subreddit;

  @Flatten("data::likes")
  private final int likes;

  @Flatten("data::view_count")
  private final int viewCount;

  @Flatten("data::title")
  private final String title;

  @Flatten("data::score")
  private final int score;

  @Flatten("data::hidden")
  private final boolean hidden;

  @Flatten("data::post_hint")
  private final String postHint;

  @Flatten("data::permalink")
  private final String permalink;

  @Flatten("data::url")
  private final String url;

  @Flatten("data::subreddit_type")
  private final String subredditType;

  @Flatten("data::hide_score")
  private final boolean hideScore;

  @Flatten("data::quarantine")
  private final boolean quarantine;

  public boolean isUnsafe() {
    return isHidden()
        || isQuarantine()
        || (getBannedBy() != null && !getBannedBy().equals(""))
        || getScore() < 1
        || !getSubredditType().equalsIgnoreCase("public")
        || !isUrlSafe();
  }

  public boolean isSafe() {
    return !isUnsafe();
  }

  public boolean isUrlSafe() {
    return url != null
        && !url.isEmpty()
        && url.startsWith("https://i.redd.it/")
        && Functions.endsWithAny(
        url,
        true,
        ".jpg", ".jpeg", ".png", ".gif", ".gifv", ".webm"
    );
  }

  public RedditPost(
      String bannedBy,
      String subreddit,
      int likes,
      int viewCount,
      String title,
      int score,
      boolean hidden,
      String postHint,
      String url,
      String permalink,
      String subredditType,
      boolean hideScore,
      boolean quarantine
  ) {
    this.bannedBy = bannedBy;
    this.subreddit = subreddit;
    this.likes = likes;
    this.viewCount = viewCount;
    this.title = title;
    this.score = score;
    this.hidden = hidden;
    this.postHint = postHint;
    this.url = url;
    this.permalink = permalink;
    this.subredditType = subredditType;
    this.hideScore = hideScore;
    this.quarantine = quarantine;
  }

  public String getBannedBy() {
    return bannedBy;
  }

  public String getSubreddit() {
    return subreddit;
  }

  public int getLikes() {
    return likes;
  }

  public int getViewCount() {
    return viewCount;
  }

  public String getTitle() {
    return title;
  }

  public int getScore() {
    return score;
  }

  public boolean isHidden() {
    return hidden;
  }

  public String getPostHint() {
    return postHint;
  }

  public String getUrl() {
    return url;
  }

  public String getPermalink() {
    return permalink;
  }

  public String getSubredditType() {
    return subredditType;
  }

  public boolean isHideScore() {
    return hideScore;
  }

  public boolean isQuarantine() {
    return quarantine;
  }

  @Override
  public String toString() {
    return new StringJoiner(", ", RedditPost.class.getSimpleName() + "[", "]")
        .add("bannedBy='" + bannedBy + "'")
        .add("subreddit='" + subreddit + "'")
        .add("likes=" + likes)
        .add("viewCount=" + viewCount)
        .add("title='" + title + "'")
        .add("score=" + score)
        .add("hidden=" + hidden)
        .add("postHint='" + postHint + "'")
        .add("permalink='" + permalink + "'")
        .add("url='" + url + "'")
        .add("subredditType='" + subredditType + "'")
        .add("hideScore=" + hideScore)
        .add("quarantine=" + quarantine)
        .add("unsafe=" + isUnsafe())
        .add("safe=" + isSafe())
        .add("urlSafe=" + isUrlSafe())
        .toString();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof RedditPost)) {
      return false;
    }
    RedditPost that = (RedditPost) o;
    return getLikes() == that.getLikes() &&
        getViewCount() == that.getViewCount() &&
        getScore() == that.getScore() &&
        isHidden() == that.isHidden() &&
        isHideScore() == that.isHideScore() &&
        isQuarantine() == that.isQuarantine() &&
        Objects.equals(getBannedBy(), that.getBannedBy()) &&
        getSubreddit().equals(that.getSubreddit()) &&
        getTitle().equals(that.getTitle()) &&
        Objects.equals(getPostHint(), that.getPostHint()) &&
        getUrl().equals(that.getUrl()) &&
        getPermalink().equals(that.getPermalink()) &&
        Objects.equals(getSubredditType(), that.getSubredditType());
  }

  @Override
  public int hashCode() {
    return Objects.hash(getBannedBy(), getSubreddit(), getLikes(), getViewCount(), getTitle(), getScore(), isHidden(),
        getPostHint(), getUrl(), getPermalink(), getSubredditType(), isHideScore(), isQuarantine());
  }

  public static class JsonResponse {
    @Flatten("data::children")
    private final List<RedditPost> posts;

    @Override
    public String toString() {
      return JsonResponse.class.getSimpleName() + "[posts=" + Functions.addCollection(
          posts,
          new StringJoiner(", ", RedditPost.class.getSimpleName() + "[", "]")
      )
          + "]";
    }

    @Override
    public boolean equals(Object o) {
      if (this == o) {
        return true;
      }
      if (!(o instanceof JsonResponse)) {
        return false;
      }
      JsonResponse that = (JsonResponse) o;
      return Objects.equals(getPosts(), that.getPosts());
    }

    @Override
    public int hashCode() {
      return Objects.hash(getPosts());
    }

    public JsonResponse(List<RedditPost> posts) {
      this.posts = posts;
    }

    public List<RedditPost> getPosts() {
      return posts;
    }
  }
}
