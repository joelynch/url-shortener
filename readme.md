# Url shortener

A simple URL shortener in rust with 2 endpoints:

1. Create a shortened version of a URL

```http
POST /shorten

{
  "url": "https://www.google.com"
}
```

This returns

```json
{
  "url": "https://www.google.com",
  "short_url": "http://localhost:3000/s/nS0tV6jX",
  "code": "nS0tV6jX"
}
```

Performing a get request on the returned `short_url` `http://localhost:3000/s/nS0tV6jX` will now redirect to the `url` field.

2. Return stats on the number of requests for a shortened url and the time of the most recent request.

```http
GET /stats/nS0tV6jX
```

Note that shortening the same url twice will return a different short url with different stats tracking.

## Running

Run with docker compose

```bash
docker-compose build
docker-compose up -d
```

After starting the database container, you will need to run database migrations.  This can be done with the `sqlx` cli as follows:

```bash
cargo install sqlx-cli
sqlx migrations run
```

Otherwise, for quick setup if you don't want to install sqlx, you could set up the database by manually running the migrations file, eg

```bash
PGPASSWORD=postgres psql -h localhost -p 5432 -U url-shortener -d url-shortener -f migrations/20230519123321_init.up.sql
```

Run tests as follows

```bash
docker-compose up -d db
export 'DATABASE_URL=postgres://url-shortener:postgres@localhost:5432/url-shortener'
cargo test
```

## Architecture

Besides an annoyingly large amount of boilerplate, this app is very simple.  In order to generate a short url, we take a hash (sha256 in this case) of the full url, and then truncate the base64 encoded version to 8 characters (enough for collisions to be very very rare).  There are 2 issues with this.  Firstly, even truncated to 8 characters, it is possible for there to be collisions where two different urls shorten to the same value.  Additionally, we want the same url to be able to be shortened multiple times to different short urls with different stats tracking.

The solution here is to put a unique index on the short url column.  When shortening a url, we maintain a counter and append to the end of the url before shortening.  If the unique constraint fails, increment the counter and try again until there is a success.

Hits tracking is very simple, the `hits` table is just an append only table that has a row inserted every time a shortened url is visited.

### Things to consider in the future

* Explore other methods of shortening the url.  I have tried to be relatively generic with the `ShorteningStrategy` enum so this should be easy.  One interesting option would be a "git-like" hash truncating strategy where we truncate to different lengths if there is a conflict.
* If lots of people want to shorten the same url, we might end up looping for a long time while incrementing the counter to find a value that succeeds.  An alternative might be to generate a random counter value each time.
* The unique index is a bit of a bottleneck for the database.

## Libraries

* Axum: web framework
* sqlx: database access
* serde: json serialization
* tracing: logging
* anyhow: error handling
