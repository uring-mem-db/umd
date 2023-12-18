.PHONY: build-docker-dev
build-docker-dev:
	@docker build -t umd-dev -f docker/Dockerfile-dev .

.PHONY: run-docker-dev
run-docker-dev:
	@docker run -it -v $(PWD):/home umd-dev

