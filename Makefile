watch-test:
	cargo watch -x check -x test

watch-run:
	cargo watch -x check -x run

docker-build:
	docker image build --tag zero2prod --file Dockerfile .

docker-run:
	docker container run -d --rm --name zero2prod -p 8080:8080 zero2prod

log-check:
	export RUST_LOG="sqlx=error,debug"; \
	export TEST_LOG=enabled; \
	cargo t subscribe_fails_if_there_is_a_fatal_database_error | bunyan