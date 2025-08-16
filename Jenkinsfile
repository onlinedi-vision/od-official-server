pipeline {
  agent {
	  label 'docker'
  }
  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {
	  agent {
		  dockerfile true
	  }
	  
	  stage('Docker Kill') {
		  steps {
			  sh 'docker kill backend_container || echo "NO ALIVE CONTAINER"'
			  sh 'docker rm backend_container || echo "NO CONTAINER NAMED backend_container"'
		  }
	  }

	  stage('Docker Build') {
      steps {
		  sh 'docker build -t api .'
      }
	  }
    stage('Docker Run') {
      steps {
        withCredentials([vaultString(credentialsId:'vault-scylla-cassandra-password',variable:'SCYLLA_CASSANDRA_PASSWORD')]){
		      sh 'docker run -d -p 127.0.0.1:1313:1313 --name backend_container --env SCYLLA_CASSANDRA_PASSWORD=$SCYLLA_CASSANDRA_PASSWORD --env WS_PORT="9002" --env API_PORT="1313" api:latest'
        }
      }
    }
  }
}
