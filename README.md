# holysee

[![Build Status](https://travis-ci.org/crisidev/holysee.svg?branch=master)](https://travis-ci.org/crisidev/holysee)

Thought out badly, understood worse and written the worst, this should be a telegram<->irc
relay bot with a reasonable configuration

**DISCLAIMER**: anything that can ever come from this package is pure garbage. Only a madman
would ever learn or study this package. You have been warned.

## Dependencies

This abomination uses the excellent [telegram-bot](https://github.com/telegram-rs/telegram-bot/) library by vendoring
a copy of it - to avoid depending on the latest git - in the `vendor/` directory. Many thanks to the original
authors for their excellent job.

# Usage

Build the bot and run it from where it can access the data dir configured in the toml file.

## Available Commands and Filters

The commands are run via any transport, using a configurable prefix, like:


```
!command param1 param2 ...
```

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

to get a specific quote you can run

- `!quote <quote_id>`

### Url Preview

The url preview command is not properly a command, in the sense that it is not activated by user input, but instead listens
on any incoming message and parses it via regexp to extract any URLs in it. For any url it finds it will try to load it and
send the Title of the page on the channel.
