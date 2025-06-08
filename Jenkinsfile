pipeline {
	agent any

	stages {
		stage('Building') {
			steps {
				sh '. ~/export.sh; env'
				sh 'env; ps -e | grep api | cut -d" " -f4 | while read line; do kill $line; done'
				sh 'cd api; ls -alh; cargo build --release; ./target/release/api &' 
			}
		}
	}

}
