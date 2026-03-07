def version=""
pipeline {
  agent any
  
  environment {
    API_PORT='1313'
    WS_PORT='9002' 
  }

  stages {
	  stage('Push Image to Docker Registry') {
		  steps {
				script {
				
					withDockerRegistry(url: 'https://registry.onlinedi.vision:5000',  credentialsId:'docker-registry') {
						version="v" + sh(script:'echo ${GIT_BRANCH} | cut -d/ -f3- | xargs echo -n', returnStdout: true)
						echo "VERSION TO BE DEPLOYED: $version"
						sh "docker build . -t od-official-server:${version}"
						sh "docker tag od-official-server:${version} registry.onlinedi.vision:5000/od-official-server:${version}"
						sh "docker push registry.onlinedi.vision:5000/od-official-server:${version}"
					}
				}
			}
		}
		stage ('Deploying to K8S') {
			steps {
		    build job: 'PROD/K8S-INFRA/DEPLOY', parameters: [[$class: 'StringParameterValue', name: 'DEPLOYMENT', value: 'od-official-server'], [$class: 'StringParameterValue', name: 'NEW_VERSION', value: version]]
			}
		}	
  }
}
