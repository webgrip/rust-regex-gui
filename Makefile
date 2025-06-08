.PHONY: help run build release fmt clippy test docs docker-build docker-run docker-stop

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS=":.*?##"}; {printf "%-15s %s\n", $$1, $$2}'

run: ## Run the application
	cargo run

build: ## Build in debug mode
	cargo build

release: ## Build in release mode
	cargo build --release

fmt: ## Format the code
	cargo fmt

clippy: ## Run clippy lints
	cargo clippy -- -D warnings

test: ## Run tests
	cargo test

docs: ## Serve documentation locally
	mkdocs serve -f docs/techdocs/mkdocs.yml

docker-build: ## Build the docker image
	docker compose build

docker-run: ## Run the application via docker compose
	docker compose up

docker-stop: ## Stop the docker compose services
	docker compose down
