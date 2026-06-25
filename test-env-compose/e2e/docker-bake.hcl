group "default" {
  targets = [ "tests" ]
}

target "tests" {
  target = "output"
  output = [ "." ]
  network = "host"
}
