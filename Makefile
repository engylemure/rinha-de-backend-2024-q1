# Default configuration with STDOUT log output
dc-up:
	docker-compose up -d

dc-down: 
	docker-compose down

dc-build:
	docker-compose up -d --build

dc-restart: dc-down dc-up

build-image:
	docker build -t engylemure/rinha-de-backend-2024-q1 -f docker/api.Dockerfile .

push-image:
	docker push engylemure/rinha-de-backend-2024-q1