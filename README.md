# STATUS: Writing Specification. See below.

# Scored - a minimalist and safe high-score server

# Motivation
So what is this all about? Did you ever want to safely manage high-score of a game for all users? On the one hand saving the score and on the other hand showing the high-score to everyone. So this is it.

# Overview
This project consists of the server **scored**, a daemon or service. High-score can be saved via data stream. Retrieved via data stream or html. User authentication via login, password and token is supported.  
Games can be managed via **command line tool**. It connects locally to the server.  
In the game client high-score can be managed with a library or crate named **libscored**.

# Used cryptographic library'n'stuff
The idea came to me while I was thinking about high-score for my little games (still in development) and when I stumbled upon TweetNaCl: https://tweetnacl.cr.yp.to/index.html. It is a minimalist and safe cryptographic library and delivers all needed functionality for this project. The other thing was that I wanted to use the programming language Rust. The language is known to ensure at compile time that the compiled program is free of memory leaks and data races. Good for the stability of the server.

# Notation
`|` is simply a separator and not an optional.  
`u8` or `u32` is one of these datatypes sent.  
`u8x` is several of these datatypes sent.  
`\` should be in the same line but didn't fit

# Communication between command line tool and server
Before high-score can be managed, games have to be added to the server's database.  
All communication between the two is done via TCP/IP. The information is sent locally and unencrypted over the IP address 127.0.0.1.

## Managing games with the command line tool
### Add a game
This has to be done with the `GAME10` command:  
`GAME10|u8:<high-score_type>|u8:<length in bytes>|u8x:<game_name>|u32:<max_number_of_entries>`.  
It returns a token referencing the added (or existing) game:  
`TOKN10|u8x:<512 bit hash of token>`.  
It adds a unique game entry into the game table of the server's database. The entry consists of name of the game, type of high-score (number: `0` or `1`, time: `2` or `3`; see [Managing high-score information](https://github.com/FlauschBert/scored#managing-high-score-information)), generated token, name of the table holding the high-score information per user, maximum number of entries (`0` means unlimited) and time of creation. The table name is in the form `u32:<table_number>|-|<u8x:first 16 bytes of game_name padded with #>`. The timestamp is in the form `YYYY-MM-DD HH:MM:SS`.  
The maximum length of the name of the game is 256 bytes. Longer names are cut off.  
Be aware that using `0` for the number of maximum entries is dangerous because the database can overflow and block the server machine.

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
This returns a list of all game information as a stream:  
The number of all game entries first:  
`u32:<number_of_games>|\`  
For each game entry:  
`u8x:<512 bit hash of token>|\`  
`u8:<high-score_type>|\`  
`u8:<length in bytes>|u8x:<game_name>|\`  
`u32:<table_number>|u8x:<16 chars of game table name>|\`  
`u32:<max_number_of_entries>|\`  
`u8x:<timestamp of creation>|`  
If the number of games is `0` the stream ends with this information.  
For description of each entry see [Add a game](https://github.com/FlauschBert/scored#add-a-game).  

# Communication between client and server
All communication between the two is done via TCP/IP. The information (commands) are sent symmetrically encrypted with a secret session key. This session key is negotiated with Diffie-Hellmann key exchange. All algorithms are part of the TweetNaCl library.

## Malformed commands
If a malformed command is sent to the server the error is returned:  
`STAT10|666`.

## Client authentication
### User and password
#### Initial authentication and token generation
Before any high-score information can be sent to the server the client has to authenticate with user and password.  
This has to be done with the `AUTH10` command followed by username and hashed password:  
`AUTH10|u8x:<512 bit hash of password>|u8:<length in bytes>|u8x:<username>`.

The answer is a unique token generated out of user, password and random information:  
`TOKN10|u8x:<512 bit hash of token>`.  
A new token can be generated only once for a given user. If the user sends the same username and password hash combination again the token once generated is returned.  
If the same user tries to generate a token again with another password hash the command is rejected with the error:  
Error authentication request rejected: `STAT10|004`.  
The maximum length of the name of the username is 256 bytes. Longer names are cut off.

For all later operations the token is used. The token should be saved by the client and used for all operations.

A token can be created (initially or by removal and recreation) or retrieved only all 5 minutes by the client to avoid flooding the database with new entries and to avoid hitting a used password with brute force attacks. The client is identified by the IP address (peer). If the timeout is underrun the error is returned:  
Error timeout underrun: `STAT10|002`.

The server allows only a certain command queue depth at once to avoid flooding the memory. If the command queue depth is too deep the stream is dropped instantly. For the client it behaves the same as if the connection gets broken.

#### Changing the password
If the client wants to change the password of a user the command `RETH10` has to be used:  
`RETH10|u8x:<512 bit hash of token>|u8x:<512 bit hash of new password>`.  
The password can be changed all five seconds.  
The answer is a new unique token generated and returned as above in case of success.  

In case of an error the status is returned:  
Error token not found: `STAT10|001`.  
Error timeout underrun: `STAT10|002`.  
Error timeout exceeded: `STAT10|003`.

#### Removal of user and all connected information
If the client wants to remove a user it has to send the `RVUR10` command:  
`RVUR10|u8x:<512 bit hash of token>`.  
**In this case all information connected to the user is dropped too**.  
The user can be removed 5 seconds after it was created or the timestamp was updated.  
The answer is a status command `STAT10`:  
Success: `STAT10|000`.  

In case of an error the status is returned:  
Error token not found: `STAT10|001`.  
Error timeout underrun: `STAT10|002`.  
Error timeout exceeded: `STAT10|003`.

#### Changing the username
This is not allowed. The user with high-score has to stay the same all the time (in an optimum way). Authenticated user and username are exactly the same and are shown in the list of high-score as is. The encoding of the username has to be UTF-8.

#### Token lifetime
Each token generated for username and hashed password combination has a fixed lifetime of 10 minutes. After this time a new token needs to be retrieved from the server with the `AUTH10` command.

#### Data handling on the server
The server saves both username and password hash in a user table along with the generated token as primary key. Additionally IP address and two timestamps are saved. The first timestamp is updated for new authentication, token retrieval and password change. The second one marks the lifetime of the generated token.

## Managing high-score information
High-score information can be duration in seconds or a number. Best can be either shortest duration (encoded as `2`) or lowest number (encoded as `0`) or longest duration (encoded as `3`) or highest number (encoded as `1`). This is encoded in the type of high-score of the game. See [Add a game](https://github.com/FlauschBert/scored#add-a-game).

### Add or update high-score information

#### Command
This has to be done with the `SCOR10` command:  
`SCOR10|\`  
`u8x:<512 bit hash of user token>|\`  
`u8x:<512 bit hash of game token>|\`  
`u64:<high-score>`.  
High-score is interpreted as number or duration in seconds.

#### Data handling on the server
Saved are the username with high-score and timestamp of submission.
