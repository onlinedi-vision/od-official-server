pipeline {
	agent any

	stages {
		stage('Building') {
			steps {
				sh 'ps -e | awk \'{$1=$1};1\' | grep api | cut -d" " -f1 | while read line; do kill $line; done'
				sh '. ~/export.sh; cd api; cargo build --release; JENKINS_NODE_COOKIE=dontKillMe ./target/release/api > ~/rlog.logs 2> ~/errlog.logs &' 
			}
		}
	}

}
