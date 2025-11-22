# Testing

## Using the test-env

For testing purposes we use a test environment (please use it before making a PR). What does this test env do?
- creates a mirror scylla DB (similar to prod DB)
- compiles the api code (your local code)
- runs unit tests on the api code (your local code)
- runs e2e testing suite on the api code (your local code)

### How to use the test-env
For doing the above mentioned things it is sufficient to run:
```sh
./launch-test-env.sh
```

For more options checkout:
```sh
./launch-test-env.sh -h  
```

For example this command skips all tests and does not create a scylla DB locally:
```sh
./launch-test-env.sh -sS
```

Or this one the changes on which port the api runs locally:
```sh
./launch-test-env.sh -p 7777
```

There are more options use the `-h` flag to see them.

### How do I test MY api

After running `./launch-test-env.sh` you can run curls on your localhost api. Example:
```sh
curl http://localhost:1313/api/version
```

This should return the version of the API.

