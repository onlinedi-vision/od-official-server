pipeline {
  agent any

  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {

	  stage('Docker Build') {
		  steps {
				withCredentials([string(credentialsId:'vault-scylla-db-user',variable:'SCYLLA_DB_USER')]){
					withCredentials([string(credentialsId:'vault-scylla-cassandra-password',variable:'SCYLLA_CASSANDRA_PASSWORD')]){
						withCredentials([vaultString(credentialsId:'vault-aes-key',variable:'SALT_ENCRYPTION_KEY')]){
							withCredentials([vaultString(credentialsId:'vault-aes-iv',variable:'SALT_ENCRYPTION_IV')]){
								sh 'docker compose build'
							}
						}
					}
				}
			}
	  }

		stage('Docker Kill') {
			steps {
				sh 'docker compose down' 		  
			}
	  }

   	stage('Docker Run') {
			steps {
				withCredentials([string(credentialsId:'vault-scylla-db-user',variable:'SCYLLA_DB_USER')]){
					withCredentials([string(credentialsId:'vault-scylla-cassandra-password',variable:'SCYLLA_CASSANDRA_PASSWORD')]){
						withCredentials([vaultString(credentialsId:'vault-aes-key',variable:'SALT_ENCRYPTION_KEY')]){
							withCredentials([vaultString(credentialsId:'vault-aes-iv',variable:'SALT_ENCRYPTION_IV')]){
		        		sh 'docker compose up -d'
							}
						}
					}
				}
			}
		}
  }
}
