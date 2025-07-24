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
        stage('Build WS') {
          steps {
            sh 'cd ws; cargo build --release;'
          }
        }
        
        stage('Run') {
          environment {
            SCYLLA_CASSANDRA_PASSWORD = credentials('scylla-password')
	        }
          parallel {
            stage('Run WS') {
              steps {
                sh 'export WS_PORT="9002";JENKINS_NODE_COOKIE=dontKillMe ./target/release/ws > ~/wslog.logs 2> ~/wselog.logs &' 
              }
            }
            
            stage('Build Docker API') {
              steps {
                sh 'mv ~/cdn ./cdn '
                sh 'docker build -t api .'
                sh 'docker run -d -p 1313:1313 --env SCYLLA_CASSANDRA_PASSWORD=$SCYLLA_CASSANDRA_PASSWORD --env API_PORT="1313" api:latest'
                sh 'mv ./cdn ~/cdn'
              }
            }
      }
      
  }
}
}
