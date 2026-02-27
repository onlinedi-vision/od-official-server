# Division Online Official Server
## Description
This is the "RESTful" API server of the Online Division project. In the near future this will also be the API server used by "self-hosters".
## Usage
Obviously, to be able to use the source code of the project you will have to first clone it:
```sh
$ git clone 'https://github.com/onlinedi-vision/od-official-server.git'
```
For the API server to properly run you also need a [scyllaDB](https://www.scylladb.com/) instance to which the server should connect to and use it as (well obviously) its data base.

You can do this manually, but more easily you can run the following command:
```sh
./launch-test-env.sh -sS
```
This command will run a docker container containing a scylla instance, and will compile the source code of od-official-server and run it connecting the two.

For more in depth analysis of what it does please either check out the code of `launch-test-env` or see the help message of the command:
```sh
./launch-test-env.sh -h
```

## Environment Variables

The "Test Env" sets up some random credentials (such as keys for encryption, and so on...). To have better control of what the keys are you can edit `launch-test-env.sh`, specifically these lines:
```sh
export SALT_ENCRYPTION_IV="ffA_1D6s^jf!6\$xx"
export SALT_ENCRYPTION_KEY='#a1aA3!h4a@ah3a4'
export SCYLLA_CASSANDRA_PASSWORD='cassandra'
export API_PORT=1313
export NO_OF_WORKERS=32
export EXECUTABLE_NAME="api"
```
These being the lines that set up the necessary environment variables for `od-official-server` to run successfuly.

## Contributing

Please checkout [CONTRIBUTING.md](./CONTRIBUTING.md) and [TESTING.md](./TESTING.md).

