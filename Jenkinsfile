pipeline {
  agent any
  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {
	  
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
        	sh 'docker compose up -d'
					}
      	}
	}
  }
}
