pipeline {
  agent any
  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {
	  
	  stage('Docker Kill') {
		  steps {
				sh 'docker compose down' 		  
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
