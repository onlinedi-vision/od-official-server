pipeline {
  agent any
  
  environment {
    API_PORT = '1313'
    WS_PORT = '9002' 
  }

  stages {
    stage('Kill WS processes') {
      steps {
        script {
          // Safer process killing with proper grep patterns
          sh '''
            pids=$(ps -e -o pid,comm | awk '/[w]s/ {print $1}')
            [ -z "$pids" ] || echo "$pids" | xargs kill -9
          '''
        }
      }
    }
    
    stage('Run') {
      parallel {
        stage('Build Docker API & WS') {
          steps {
            // Proper Vault credential binding with error handling
            script {
              withCredentials([
                string(
                  credentialsId: 'vault-scylla-cassandra-password',
                  variable: 'SCYLLA_CASSANDRA_PASSWORD'
                )
              ]) {
                try {
                  // Directory operations with checks
                  sh '''
                    [ -d ./cdn ] || (mkdir -p ~/cdn && mv ~/cdn ./cdn 2>/dev/null || true)
                    docker build -t api .
                    docker run -d \
                      -p 127.0.0.1:${API_PORT}:${API_PORT} \
                      -e SCYLLA_CASSANDRA_PASSWORD=${SCYLLA_CASSANDRA_PASSWORD} \
                      -e WS_PORT=${WS_PORT} \
                      -e API_PORT=${API_PORT} \
                      --name api_container \
                      api:latest
                    [ -d ./cdn ] && mv ./cdn ~/cdn
                  '''
                } catch (Exception e) {
                  error("Failed to build/run container: ${e.getMessage()}")
                }
              }
            }
          }
        }
      }
    }
  }
}
