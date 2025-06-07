pipeline {
	agent any

	stages {
		stage('Building') {
			steps {
				echo 'here'
				sh 'ls -lah'
				sh 'cd api; ls -alh; cargo build --release; ./target/release/api'
			}
		}
	}

}
