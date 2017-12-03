# Holysee

[![Build Status](https://travis-ci.org/crisidev/holysee.svg?branch=master)](https://travis-ci.org/crisidev/holysee)
[![codecov](https://codecov.io/gh/crisidev/holysee/branch/master/graph/badge.svg)](https://codecov.io/gh/crisidev/holysee)

Thought out badly, understood worse and written the worst, this should be a telegram<->irc
relay bot with a reasonable configuration.

**DISCLAIMER**: anything that can ever come from this package is pure garbage. Only a madman
would ever learn or study this package. You have been warned.

## Dependencies

This abomination uses the excellent [telegram-bot](https://github.com/telegram-rs/telegram-bot/) library by vendoring
a copy of it - to avoid depending on the latest git - in the `vendor/` directory. Many thanks to the original
authors for their excellent job.

# Configuration

The provided configuration file is pretty self explanatory, just copy it to `config/local.toml` for it to be loaded
by the bot during startup.

To convert irc nicknames to telegram and viceversa you can configure the `[[nicknames]]` map in the toml file as follows:

```
[[nickanmes]]
irc = "user1"
telegram = "@someother"
[[nicknames]]
irc = "user2"
telegram = "@user2"
```

In this example remember to add the `@` character before the username of the telegram user, or the translation will fail.

There is currently no way of disabling this feature, feel free to find a way of configuring without breaking :)

Enabled commands can be configured as well:

```
[commands]
data_dir = "./data"
enabled = [
  "karma",
  "quote",
  "last_seen",
  "url_preview",
]
```

# Usage

## Stable-ish version
Check [Release](https://github.com/crisidev/holysee/releases) page, download the prebuild binary (Linux only) and run it with a proper configurations structure:

```
tar xfvz holysee-$version-linux-amd64.tar.gz
cd holysee
edit config/local.toml
RUST_LOG=holysee=info ./holysee
```

## Git version
Build the bot and run it from where it can access the data dir configured in the toml file:

```
git clone https://github.com/crisidev/holysee
cd holysee
make
cd holysee
cp config/example.toml config/local.toml
edit config/local.toml
RUST_BACKTRACE=1 RUST_LOG=holysee=debug ./target/debug/holysee
```

## Available Commands and Filters
* karma
* last_seen
* quote
* url_preview

The commands are run via any transport, using a configurable prefix, like:

```
!command param1 param2 ...
```

### Help / Usage

Every command is provided with a self-explaining help which will be sent, where possible, as private message to the requester.
**It is always enabled.**

```
!help command
!usage command
```

Note: `!usage` and `!help` are aliased.

### Relay

Relay is a special command allowing IRC <-> Telegram relay. **It is always enabled and triggered on every message.**

### Karma

Karma records string karma for posterity. To create (or increment) the karma of any string you can run:

- `viva <string>` or `<string>++`

To decrement it

- `abbasso <string>` or `<string>--`

To view the karma for a string:

- `!karma <string>`

### Last Seen

The last seen command maintains a timestamp of the last time a user has written some message in the channel. You can see
the last time a nickname has said something by running:

- `!seen <nickname>`

### Quote

The quote command maintains a list of quotes. To get a random quote run

- `!quote`

to add a quote use

- `!quote add <string>`

to delete a quote use

- `!quote rm <quote_id>`
- `!quote rm <string>`

to get a specific quote you can run

- `!quote <quote_id>`

### Url Preview

The url preview command is not properly a command, in the sense that it is not activated by user input, but instead listens
on any incoming message and parses it via regexp to extract any URLs in it. For any url it finds it will try to load it and
send the Title of the page on the channel.
