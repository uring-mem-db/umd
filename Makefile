.PHONY: build-docker
build-docker:
	@docker build -t umd .

.PHONY: run-docker
run-docker:
	@docker run -it -v $(PWD):/home umd
