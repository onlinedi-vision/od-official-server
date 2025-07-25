pipeline {
  agent any
  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {
        stage('Killing') {
          parallel {
            stage('Kill API processes') {
              steps {
                sh 'ps -e | awk \'{$1=$1};1\' | grep api | cut -d" " -f1 | while read line; do kill $line; done'
              }
            }
        
            stage('Kill WS processes') {
              steps {
                sh 'ps -e | awk \'{$1=$1};1\' | grep ws | cut -d" " -f1 | while read line; do kill $line; done'
              }
            }
          }
        } 
        stage('Run') {
          environment {
            SCYLLA_CASSANDRA_PASSWORD = credentials('scylla-password')
	  }
	  parallel {
	    stage('Build Docker API & WS') {
	      steps {
		sh '[ -d ./cdn ] || mv ~/cdn ./cdn'
		sh 'docker build -t api .'
		sh 'docker run -dit -p 127.0.0.1:1313:1313 --env SCYLLA_CASSANDRA_PASSWORD=$SCYLLA_CASSANDRA_PASSWORD --env WS_PORT="9002" --env API_PORT="1313" api:latest'
		sh '[ -d ./cdn ] && mv ./cdn ~/cdn'
	      }
	    }
  	 }
	}
  }
}
