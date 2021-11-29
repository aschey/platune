set dotenv-load := false

build *ARGS:
    #!/usr/bin/env sh
    cargo build {{ARGS}}
    (cd ./platune-cli && go build) 

setup:
    npm install
    npm run prepare
    go install github.com/golang/mock/mockgen@v1.6.0
    go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.27.1
    go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.1
    curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(go env GOPATH)/bin v1.43.0

test *ARGS:
    #!/usr/bin/env sh
    cargo test --locked --features=dummy -- {{ARGS}}
    (cd ./platune-cli && go test ./...)    

lint:
    cargo clippy
    (cd ./platune-cli && golangci-lint run)

regen-grpc:
    #!/usr/bin/env sh
    scripts/regen-grpc.sh

server *ARGS:
    #!/usr/bin/env sh
    (cd ./platuned/server && cargo run {{ARGS}})

cli *ARGS:
    #!/usr/bin/env sh
    (cd ./platune-cli && go run . {{ARGS}}) 

win-srv:
    (cd ./platuned/server && cargo run --release -- -i)

stop-win-srv:
    net stop platuned
    
systemd:
    cp ./platuned/linux/platuned.service /etc/systemd/system/platuned.service
    systemctl daemon-reload
    systemctl restart platuned

verify-features:
    cargo hack --feature-powerset --exclude-no-default-features clippy --locked -- -D warnings