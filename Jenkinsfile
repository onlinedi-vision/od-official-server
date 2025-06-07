pipeline {
	agent any

	stages {
		stage('Building') {
			steps {
				echo 'here'
				sh 'ls -lah'
				sh 'ps -e | grep api | cut -d" " -f4 | while read line; do echo $line; done'
				sh 'cd api; ls -alh; cargo build --release; ./target/release/api'
			}
		}
	}

}
