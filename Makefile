watch:
	cargo watch -x check -x test -x run

docker-build:
	docker image build --tag zero2prod --file Dockerfile .

docker-run:
	docker container run --rm --name zero2prod -p 8080:8080 zero2prod