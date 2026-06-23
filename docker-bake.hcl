variable "GIT_BRANCH" {}

group "default" {
  targets = [ "release" ]
}

function "tag" {
  params = [ branch ]
  result = "v${split("/", branch)[2]}"
}

target "release" {
  target = "runtime"
  contexts = {
    bare-repo = "test-env-compose/rootfs/repo"
  }
  args = {
    GIT_BRANCH = "${GIT_BRANCH}"
  }
  output = [ "type=cacheonly" ]
  tags = [ "registry.onlinedi.vision:5000/od-official-server:${tag("${GIT_BRANCH}")}" ]
}
