# Youtube Server

This project takes [piped](https://github.com/TeamPiped/Piped) and makes it easy to run. Click on the binary to run. That's it!*

* Requires java to be installed (and on PATH), and [PostgreSQL](https://www.postgresql.org/download/) with configured a db

## Config
On the first run, a config will be generated in the same folder as the executable. Alter it to your liking. For more options, please see [config.rs](src/config.rs)

## Running
To run this, you need Java installed (and on PATH). You also need to install [PostgreSQL](https://www.postgresql.org/download/), and configure a server for the db connection

## Building

You need node and `pnpm` installed first (and in your PATH). You also need [Rust installed](https://rustup.rs/) as well as java installed (and on the PATH)
- Run `cargo build --release`

## FAQ
### My custom instance isn't updating to a new url!
Check and clear your browsers local storage. It likes to save the custom instance in there. Also clear your browser cache completely just to make sure.

### How do I make a postgresql database for this?
Open pgadmin, make a new database called "piped" (you can choose any name really), a new user (make sure to allow them to login), and set user privileges to All for that db. Here's a [tutorial](http://youtu.be:8080/watch?v=oNJpktM65eY). After, just update the connection string with your ip (usually localhost), username, and password you just made

### I don't want to use postgresql! Is there an alternative?
It is possible for you to use hsqldb, but this only receives limited testing and is not guaranteed to work, so you use it at your own risk. Use the following values in the config
```toml
db_connection_url = "jdbc:hsqldb:mem:memdb;sql.syntax_pgs=true"
db_connection_driver = "org.hsqldb.jdbcDriver"
db_dialect = "org.hibernate.dialect.HSQLDialect"
```
If you want a persistent db (instead of losing it when the process dies), you could try:
```toml
db_connection_url = "jdbc:hsqldb:file:/file/path/to/your.db;sql.syntax_pgs=true"
```
