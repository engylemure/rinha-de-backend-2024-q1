# Default configuration with STDOUT log output
dc-up:
	docker-compose up -d

dc-down: 
	docker-compose down

dc-build:
	docker-compose up -d --build

dc-restart: dc-down dc-up