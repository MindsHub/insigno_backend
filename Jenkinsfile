pipeline {

  def remote = [: ]
  remote.name = 'insignio-server'
  remote.host = '35.156.179.141'
  remote.user = 'mattia'
  remote.password = '${SH_PASS}'
  remote.allowAnyHosts = true

  agent any
  stages {
    stage('Hello!') {
        steps {
          echo 'Ciao mondo!'
        }
      }
      stage('Remote SSH') {
        steps {}
        writeFile file: 'abc.sh', text: 'ls -lrt'
        //sshScript remote: remote, script: "abc.sh"
      }

  }
}