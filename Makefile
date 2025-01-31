# Variables
USER ?= "$$(whoami)"
DOCKER_IMAGE ?= "reporteer"
DOCKER_TAG ?= "latest"
PLATFORM ?= linux/amd64
PORT ?= 3000
CARGO_TARGET_DIR := target
GIT_HASH ?= "$$(git show-ref --head --abbrev=8 | awk '/ HEAD$$/{ print $$1 }')"

# Make 'help' the default target
.DEFAULT_GOAL := help

.PHONY: help build test clean docker-build docker-run docker-release docker-release-latest \
				docker-push docker-tag-latest docker-push-latest dev-build dev-run dev-clean

# Help target
help:  ## Display this help message
		@echo "Reporteer Makefile Help\n"
		@echo "Usage: make [target]\n"
		@echo "Available targets:"
		@awk 'BEGIN {FS = ":.*##"; printf "\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  %-40s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
		@echo ""
		@echo "Environment variables:"
		@echo "  USER          Current user (default: $$(whoami))"
		@echo "  DOCKER_IMAGE  Docker image name (default: reporteer)"
		@echo "  DOCKER_TAG    Docker tag (default: latest)"
		@echo "  PLATFORM      Build platform (default: linux/amd64)"
		@echo "  PORT          Server port (default: 3000)"
		@echo ""

# Development commands
build:  ## Build the Rust project
		cargo build --release

test:  ## Run tests
		cargo test

clean:  ## Clean build artifacts
		cargo clean
		rm -rf $(CARGO_TARGET_DIR)

# Docker commands
docker-build-no-cache:  ## Build Docker image with Git hash tag
		docker buildx build \
		--no-cache \
		--pull \
		--platform $(PLATFORM) \
		-f ./Dockerfile \
		-t "$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)" \
		./ && \
		echo "Built $(USER)/$(DOCKER_IMAGE):$(GIT_HASH)"

docker-build:  ## Build Docker image with Git hash tag
		docker buildx build \
		--pull \
		--platform $(PLATFORM) \
		-f ./Dockerfile \
		-t "$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)" \
		./ && \
		echo "Built $(USER)/$(DOCKER_IMAGE):$(GIT_HASH)"

docker-run:  ## Run Docker container
		docker run -p $(PORT):$(PORT) \
		-e RUST_LOG=info \
		-e REPORTEER_SERVER_PORT=$(PORT) \
		-e REPORTEER_ENDPOINT_URL=http://127.0.0.1:8006/derived_key \
		"$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)"

docker-release: docker-build docker-push  ## Build and push Docker image with Git hash tag
docker-release-latest: docker-release docker-tag-latest docker-push-latest  ## Build and push Docker image with both Git hash and latest tags

docker-push:  ## Push Docker image with Git hash tag
		docker push "$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)" && \
		echo "Pushed $(USER)/$(DOCKER_IMAGE):$(GIT_HASH)"

docker-tag-latest:  ## Tag Docker image as latest
		docker tag \
		"$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)" \
		"$(USER)/$(DOCKER_IMAGE):latest" && \
		echo "Tagged $(USER)/$(DOCKER_IMAGE):$(GIT_HASH) as $(USER)/$(DOCKER_IMAGE):latest"

docker-push-latest:  ## Push Docker image with latest tag
		docker push "$(USER)/$(DOCKER_IMAGE):latest" && \
		echo "Pushed $(USER)/$(DOCKER_IMAGE):latest"

# Development targets
dev-build:  ## Build the project locally using cargo
		cargo build

dev-run: dev-build  ## Build and run the project locally
		cargo run

dev-clean:  ## Clean up build artifacts and Docker images
		cargo clean
		docker rmi -f "$(USER)/$(DOCKER_IMAGE):$(GIT_HASH)" "$(USER)/$(DOCKER_IMAGE):latest" 2>/dev/null || true

# vim: set filetype=make foldmethod=marker foldlevel=0 noexpandtab:
