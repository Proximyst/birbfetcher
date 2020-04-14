package com.proximyst.birbfetcher;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.moandjiezana.toml.Toml;
import com.moandjiezana.toml.TomlWriter;
import com.proximyst.birbfetcher.api.RedditApi;
import com.proximyst.birbfetcher.api.RedditPost;
import com.proximyst.birbfetcher.http.RandomImage;
import com.proximyst.birbfetcher.tasks.PostFetchTask;
import com.proximyst.birbfetcher.tasks.PostProcessTask;
import com.proximyst.birbfetcher.utils.Functions;
import com.proximyst.birbfetcher.utils.TimeMeasurer;
import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import java.io.BufferedOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;
import java.sql.SQLException;
import java.util.Optional;
import java.util.Queue;
import java.util.Timer;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.ThreadFactory;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import okhttp3.Dispatcher;
import okhttp3.OkHttpClient;
import org.itishka.gsonflatten.FlattenTypeAdapterFactory;
import org.jetbrains.annotations.NotNull;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import retrofit2.Retrofit;
import retrofit2.converter.gson.GsonConverterFactory;
import spark.Spark;

public class Main {
  @NotNull
  private static final Gson simpleGson = new GsonBuilder()
      .registerTypeAdapterFactory(new FlattenTypeAdapterFactory())
      .create();

  @NotNull
  private static final Logger logger = LoggerFactory.getLogger(Main.class);

  @NotNull
  private static final File birbDirectory = new File("birbs");

  @NotNull
  private final BirbFetcherConfiguration configuration;

  @NotNull
  private final OkHttpClient httpClient;

  @NotNull
  private final RedditApi redditApi;

  @NotNull
  private final ExecutorService executor;

  @NotNull
  private final HikariDataSource hikariDataSource;

  @NotNull
  private final Queue<RedditPost> postQueue = new LinkedBlockingQueue<>();

  public Main(
      @NotNull BirbFetcherConfiguration configuration,
      @NotNull OkHttpClient httpClient,
      @NotNull RedditApi redditApi,
      @NotNull ExecutorService executor,
      @NotNull HikariDataSource hikariDataSource
  ) {
    this.configuration = configuration;
    this.httpClient = httpClient;
    this.redditApi = redditApi;
    this.executor = executor;
    this.hikariDataSource = hikariDataSource;
  }

  @SuppressWarnings("ResultOfMethodCallIgnored")
  public static void main(String[] args) throws IOException {
    birbDirectory.mkdirs();

    var timeMeasurer = new TimeMeasurer();

    timeMeasurer.start();
    BirbFetcherConfiguration config;
    File configFile = new File("config.toml");
    if (configFile.isFile()) {
      logger.info("Loading config from config.toml...");
      logger.debug("Absolute config path: " + configFile.getAbsolutePath());
      var toml = new Toml().read(configFile);
      config = toml.to(BirbFetcherConfiguration.class);
    } else {
      logger.info("Creating default config in config.toml...");
      configFile.delete();
      Optional.ofNullable(configFile.getParentFile())
          .ifPresent(File::mkdirs);

      config = new BirbFetcherConfiguration(
          "https://reddit.com",
          "Mozilla/5.0",
          "jdbc:mariadb://localhost:3306/birbfetcher",
          "root",
          "root",
          Stream.of("parrots", "birb", "birbs").collect(Collectors.toSet()),
          8
      );
      var toml = new TomlWriter().write(config);

      Files.write(
          configFile.toPath(),
          toml.getBytes(),
          StandardOpenOption.CREATE,
          StandardOpenOption.TRUNCATE_EXISTING,
          StandardOpenOption.WRITE
      );

      logger.info("Now that a configuration exists in config.toml, please edit it before starting the app again.");
      return;
    }
    logger.info("Reading config took " + timeMeasurer.stop() + "ms.");

    timeMeasurer.start();
    logger.info("Creating thread pool...");
    ThreadFactory threadFactory = (runnable) -> {
      var thread = new Thread(runnable);
      thread.setDaemon(true);
      return thread;
    };
    ExecutorService threadPool;
    if (config.getThreadPoolSize() > 0) {
      logger.info("Creating thread pool using " + config.getThreadPoolSize() + " threads.");
      threadPool = Executors.newFixedThreadPool(config.getThreadPoolSize(), threadFactory);
    } else {
      logger.info("Creating thread pool using any amount of threads.");
      threadPool = Executors.newCachedThreadPool(threadFactory);
    }
    logger.info("Creating thread pool took " + timeMeasurer.stop() + "ms.");

    logger.info("Creating the HTTP API client...");
    timeMeasurer.start();
    logger.debug("Creating OkHttpClient...");
    var httpClient = new OkHttpClient.Builder()
        .cache(null)
        .connectTimeout(5, TimeUnit.SECONDS)
        .followRedirects(true)
        .dispatcher(new Dispatcher(threadPool))
        .addInterceptor(chain -> chain.proceed(
            chain.request().newBuilder()
                .addHeader("User-Agent", config.getUserAgent())
                .build()
        ))
        .build();

    logger.debug("Creating Retrofit...");
    var retrofit = new Retrofit.Builder()
        .addConverterFactory(GsonConverterFactory.create(simpleGson))
        .baseUrl(config.getApiBaseUrl())
        .client(httpClient)
        .build();

    logger.debug("Creating RedditApi...");
    var redditApi = retrofit.create(RedditApi.class);
    logger.info("Creating the HTTP API client took " + timeMeasurer.stop() + "ms.");

    logger.info("Connecting to SQL...");
    timeMeasurer.start();
    var hikariConfig = new HikariConfig();
    hikariConfig.setJdbcUrl(config.getJdbc());
    hikariConfig.setUsername(config.getSqlUsername());
    hikariConfig.setPassword(config.getSqlPassword());
    hikariConfig.setThreadFactory(threadFactory);
    var hikari = new HikariDataSource(hikariConfig);
    logger.info("Connecting to SQL took " + timeMeasurer.stop() + "ms.");

    logger.debug("Calling Main#run...");
    new Main(
        config,
        httpClient,
        redditApi,
        threadPool,
        hikari
    ).run();
  }

  private void run() {
    var timeMeasurer = new TimeMeasurer();
    try (var connection = getHikariDataSource().getConnection();
        var checkImagesStmt = connection.createStatement();
        var createBirbsStmt = connection.createStatement();
        var createHashIndexStmt = connection.createStatement();
    ) {
      timeMeasurer.start();
      try (var hasImageColumn = checkImagesStmt.executeQuery("SHOW COLUMNS FROM birbs LIKE 'image'")) {
        if (hasImageColumn.first()) {
          // Have to migrate data first!
          try (var getImagesStmt = connection.createStatement();
              var result = getImagesStmt.executeQuery("SELECT hash, image FROM birbs");
              var dropColumnStmt = connection.createStatement()) {
            while (result.next()) {
              var hash = Functions.bytesToHex(
                  Functions.readEntireStream(
                      result.getBinaryStream("hash")
                  )
              );
              var image = Functions.readEntireStream(
                  result.getBlob("image").getBinaryStream()
              );

              var file = new File(getBirbDirectory(), hash);
              if (file.exists()) {
                Files.delete(file.toPath());
              }
              file.createNewFile();
              try (var stream = new FileOutputStream(file);
                  var bufferedStream = new BufferedOutputStream(stream)) {
                bufferedStream.write(image);
              }
            }

            dropColumnStmt.executeUpdate("ALTER TABLE birbs DROP COLUMN image");
          } catch (IOException ex) {
            logger.error("Cannot migrate images!", ex);
            return;
          }
        }
      }
      logger.info("Migration procedure took " + timeMeasurer.stop() + "ms.");
      timeMeasurer.start();

      createBirbsStmt.executeUpdate(
          "CREATE TABLE IF NOT EXISTS birbs ("
              + "id INT NOT NULL AUTO_INCREMENT,"
              + "hash BINARY(32) NOT NULL,"
              + "permalink TINYTEXT NOT NULL,"
//              + "image MEDIUMBLOB NOT NULL," // Now uses files.
              + "source_url VARCHAR(512) NOT NULL,"
              + "content_type VARCHAR(64) NOT NULL,"
              + "banned BOOL NOT NULL DEFAULT false,"
              + "PRIMARY KEY (id),"
              + "UNIQUE (hash)"
              + ")"
      );

      try {
        createHashIndexStmt.executeUpdate(
            "ALTER TABLE birbs ADD UNIQUE INDEX hash_uidx (hash)"
        );
      } catch (SQLException ex) {
        // Error Code: 1061. Duplicate key name 'hash_uidx';
        // We don't care. It's only once a run, so it's fine.
        if (ex.getErrorCode() != Constants.DUPLICATE_KEY_NAME_ERROR_CODE) {
          throw ex;
        }
      }
      logger.info("Creating tables and indices took " + timeMeasurer.stop() + "ms.");
    } catch (SQLException ex) {
      logger.error("Could not setup SQL.", ex);
      return;
    }

    var timer = new Timer();
    var postFetch = new PostFetchTask(this);
    var postProcess = new PostProcessTask(this);

    timer.scheduleAtFixedRate(postFetch, 0, TimeUnit.MINUTES.toMillis(10));
    timer.scheduleAtFixedRate(postProcess, TimeUnit.MINUTES.toMillis(1), TimeUnit.MINUTES.toMillis(10));

    Spark.port(80);
    var randomImageEndpoint = new RandomImage(this);
    {
      var cores = Runtime.getRuntime().availableProcessors();
      if (cores > 16) {
        cores = (int) Math.ceil((double) cores / 2);
      } else {
        cores = 12;
      }
      Spark.threadPool(cores);
      logger.info("Using " + cores + " threads for Spark.");
    }
    Spark.get(
        "/",
        randomImageEndpoint
    );
    Spark.notFound(randomImageEndpoint);

    Runtime.getRuntime().addShutdownHook(new Thread(() -> {
      timer.cancel();
      Spark.stop();
    }));
  }

  @NotNull
  public static Gson getSimpleGson() {
    return simpleGson;
  }

  @NotNull
  public static File getBirbDirectory() {
    return birbDirectory;
  }

  @NotNull
  public BirbFetcherConfiguration getConfiguration() {
    return configuration;
  }

  @NotNull
  public OkHttpClient getHttpClient() {
    return httpClient;
  }

  @NotNull
  public RedditApi getRedditApi() {
    return redditApi;
  }

  @NotNull
  public ExecutorService getExecutor() {
    return executor;
  }

  @NotNull
  public Queue<RedditPost> getPostQueue() {
    return postQueue;
  }

  @NotNull
  public HikariDataSource getHikariDataSource() {
    return hikariDataSource;
  }
}
