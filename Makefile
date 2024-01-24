# default `make` prints help
.PHONY: help
help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build-docker-dev    Build docker image for development"
	@echo "  run-docker-dev      Run docker image for development"

.PHONY: build-docker-dev
build-docker-dev:
	@docker build -t umd-dev -f docker/Dockerfile-dev .

.PHONY: run-docker-dev
run-docker-dev:
	@docker run -it -v $(PWD):/home umd-dev

