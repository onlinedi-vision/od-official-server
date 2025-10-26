# Division Online Official Server
## Description
This is the server that will host the global user data (such as the login data), official server data (encrypted messages, users, etc.) and DM's between users. The data stored on the server isn't necessarly decryptable on server side, since server owners can set flags to encrypt data symmetrically and on the client side.
## Installation
Installing the server is as easy as cloning the repo. There are some dependencies you have to keep in mind and install on your own.
```sh
$ git clone 'https://github.com/onlinedi-vision/od-official-server.git'
```
This server is written with the intention to be ran as root and installed at '/root/od-official-server'. With a logs directory and a cdn directory as described below.
Also the admin of this server must set certain Environment Variables before running the ```build.sh``` script.
A more complete installation would look like this:
```sh
$ mkdir /root/od-logs /root/cdn /root/od-official-server
$ cd /root/od-official-server
$ git clone 'https://github.com/onlinedi-vision/od-official-server.git'
```
The server's database is ScyllaDB as such this must also be installed and setup. More info about the installation can be found within ScyllaDB's [official docs](https://opensource.docs.scylladb.com/stable/getting-started/install-scylla/index.html).
## Environment Variables
Make sure to set the following environment variables before running the '''build.sh''' script.
```sh
 $ export SCYLLA_CASSANDRA_PASSWORD="the password of the 'cassandra' superuser of the scylladb instance"
```
## Usage
After installing the server and setting up the env vars and ScyllaDB, running it should be easy, from within the cloned directory:
```
$ build.sh
```
This will build and run the server.


