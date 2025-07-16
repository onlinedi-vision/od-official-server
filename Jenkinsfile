pipeline {
	agent any
  
  environment {
    API_PORT='1313'
    WS_PORT='9002'
  }

	stages {
    stage('Kill API processes') {
      steps {
				sh 'ps -e | awk \'{$1=$1};1\' | grep api | cut -d" " -f1 | while read line; do kill $line; done'
      }
    }
		stage('Build & run API') {
			steps {
				sh '. ~/export.sh;\
        if [[ $(git branch | grep shadow | wc -l) > 0 ]];\
          then export API_PORT="7777";\
        fi;\
        cd api; cargo build --release;\
        JENKINS_NODE_COOKIE=dontKillMe ./target/release/api > ~/rlog.logs 2> ~/errlog.logs &' 
			}
		}
    stage('Kill WS processes') {
      steps {
        sh 'ps -e | awk \'{$1=$1};1\' | grep ws | cut -d" " -f1 | while read line; do kill $line; done'
      }
    }
    stage('Build WS') {
      steps {
        sh 'cd ws; cargo build --release;'
      }
    }
    stage('Run WS') {
      steps {
        sh 'export WS_PORT="9002";JENKINS_NODE_COOKIE=dontKillMe ./target/release/ws > ~/wslog.logs 2> ~/wselog.logs &' 
      }
    }
	}
}
