pipeline {
  agent any
  
  stages {
	  stage('Push Image to Docker Registry') {
		  steps {
				script {
					withDockerRegistry(url: 'https://registry.onlinedi.vision:5000',  credentialsId:'docker-registry') {
						sh "docker buildx bake --set release.output=\"type=registry\""
					}
				}
			}
		}
		stage ('Deploying to K8S') {
			def version="v" + sh(script:'echo ${GIT_BRANCH} | cut -d/ -f3- | xargs echo -n', returnStdout: true)
			steps {
		    build job: 'PROD/K8S-INFRA/DEPLOY', parameters: [[$class: 'StringParameterValue', name: 'DEPLOYMENT', value: 'od-official-server'], [$class: 'StringParameterValue', name: 'NEW_VERSION', value: version]]
			}
		}	
  }
}
