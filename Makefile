watch:
	cargo watch -x check -x test -x run

docker-build:
	docker build --tag zero2prod --file Dockerfile .

docker-run:
	docker container run zero2prod