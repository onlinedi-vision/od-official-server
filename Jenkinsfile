pipeline {
	agent any

	stages {
		stage('Building') {
			steps {
				echo 'here'
				sh 'ls -lah'
				sh 'cd api'
				sh 'cargo build --release'
				sh './target/release/api'
			}
		}
	}

}
