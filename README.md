# Scored - a minimalist and safe high score server

# Motivation
So what is this all about? Did you ever want to safely manage high scores (or similar information) of a game for all users? On the one hand saving the score and on the other hand showing the score to everyone. So this is it.

# Overview
This project consists of the server **scored**, a daemon or service. High scores can be saved via data stream. Retrieved via data stream or html. User authentication via login, password and token is supported.  
Games can be managed via **command line tool** which connects locally to the server.  
In the game client high scores can be managed with a library or crate **libscored**.

# Used cryptographic library and stuff
The idea came to me while is was thinking about high scores for my little games (still in development) and when I stumbled upon TweetNaCl: https://tweetnacl.cr.yp.to/index.html. It is a minimalist and safe cryptographic library and delivers all needed functionality for this project. The other thing was that I wanted to use the programming language Rust. The language is known to ensure that the compiled program is free of data races. Good for the stability of the server.

# Notation
`|` is simply a separator and not an optional.
`u8` or `u32` is one of these datatypes sent.
`u8x` is several of these datatypes sent.

# Communication between command line tool and server
Before high scores can be managed games have to be added to the server's database.  
All communication between the two is done via TCP/IP. The information is sent not encrypted and locally over the IP address 127.0.0.1.

## Managing games with the command line tool
### Add a game
This has to be done with the `GAME10` command:  
`GAME10|u8:<highscore_type>|u8:<length in bytes>|u8x:<game_name>`.  
It returns a token referencing the added (or existing) game:  
`TOKN10|u8x:<512 bit hash of token>`.  
It adds a unique game entry into the game table of the database of the server. This entry consists of name of the game, type of high score (number: `0` or time: `1`), generated token, name of the table holding the high score information per user and time of creation. The table name is in the form `u32:<table_number>|-|<u8x:first 16 bytes of game_name padded with #>`. The timestamp is in the form `YYYY-MM-DD HH:MM:SS`.  
The maximum length of the name of the game is 256 bytes. Longer names are cut off.

### Remove a game
This has to be done with the `RVGA10` command:  
`RVGA10|u8x:<512 bit hash of token>`.  
**In this case all information connected to the game is dropped too**.

The answer is a status command `STAT10`:  
Success: `STAT10|000`.  
Error token not found: `STAT10|001`.

### List games
This has to be done with the `LIST10` command:  
`LIST10|GAME`.  
This returns a list of all game information:  
`u32:<number_of_games>|`  
`u8x:<512 bit hash of token>|u8:<highscore_type>|u8:<length in bytes>|u8x:<game_name>|u32:<table_number>|u8x:<16 chars of game table name>|u8x:<timestamp of creation>|`  
`...`.  
So this can be either `0` if no games exist yet or any number of games followed by the specific game information.

# Communication between client and server
All communication between the two is done via TCP/IP. The information (commands) has to be sent symmetrically encrypted with a secret session key negotiated with Diffie-Hellmann key exchange which is also part of the TweetNaCl library.

## Malformed commands
If a malformed command is sent to the server the error is returned:  
`STAT10|666`.

## Client authentication
### User and password
#### Initial authentication and token generation
Before any high score information can be sent to the server the client has to authenticate with a unique user and password.  
This has to be done with the `AUTH10` command followed by username and hashed password:  
`AUTH10|u8x:<512 bit hash of password>|u8:<length in bytes>|u8x:<username>`.

The answer is a unique token generated out of user, password and random information:  
`TOKN10|u8x:<512 bit hash of token>`.  
The token can be generated only once for a given user. If the user sends the same username and password hash combination as before the token once generated is returned again.  
If the same user tries to generate a token again with another password hash the command is rejected with the error:  
`STAT10|003`.  
The maximum length of the name of the username is 256 bytes. Longer names are cut off.

For all later operations the token is used. The token should be saved by the client and used for all operations.

A token can be created (initially or by removal and recreation) or retrieved only all 15 minutes by the client to avoid flooding the database with new entries and to avoid hitting a used password with brute force checks. The client is identified by the IP address (peer). If the timeout is underrun the error is returned:  
Error timeout underrun: `STAT10|002`.

The server allows only a certain command queue depth at once to avoid flooding the memory. If the command queue depth is too deep the stream is simply dropped instantly. It would be the same as if the connection gets broken.

#### Changing the password
If the client wants to change the password of a user the command `RETH10` has to be used:  
`RETH10|u8x:<512 bit hash of token>|u8x:<512 bit hash of new password>`.  
The answer is a new unique token generated and returned as above in case of success.  
In case of an error the status is returned:  
Error token not found: `STAT10|001`.

#### Removal of user and all connected information
If the client wants to remove a user it has to send the `RVUR10` command:  
`RVUR10|u8x:<512 bit hash of token>`.  
**In this case all information connected to the user is dropped too**.

The answer is a status command `STAT10`:  
Success: `STAT10|000`.  
Error token not found: `STAT10|001`.

#### Data handling on the server
The server saves both username and password hash in a user table along with the generated token as primary key. Additionally IP address and last authentication timestamp is saved per token.

## Managing high score information

