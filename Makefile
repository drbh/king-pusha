create:
	@openssl ecparam -genkey -name prime256v1 -out private_key.pem

pubkey:
	@openssl ec -in private_key.pem -pubout -outform DER|tail -c 65|base64|tr '/+' '_-'

docker-dev:
	docker buildx build \
	-t king-pusha:dev \
	-f Dockerfile .

king-pusha-app:
	flyctl apps create king-pusha
	flyctl ips allocate-v6 -a king-pusha

build-king-pusha:
	docker buildx build \
	--push -t registry.fly.io/king-pusha:latest -f Dockerfile .

deploy-king-pusha:
	flyctl m run -a king-pusha \
	--memory 256 \
	--cpus 1 \
	-p 443:8081/tcp:http:tls \
	registry.fly.io/king-pusha:latest