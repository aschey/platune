set dotenv-load := false

build *ARGS:
    #!/usr/bin/env sh
    cargo build {{ARGS}}
    (cd ./platune-cli && go build) 

test *ARGS:
    #!/usr/bin/env sh
    cargo test --features=dummy -- {{ARGS}}
    (cd ./platune-cli && go test ./...)    

regen-grpc:
    #!/usr/bin/env sh
    scripts/regen-grpc.sh

server *ARGS:
    #!/usr/bin/env sh
    (cd ./platuned/server && cargo run {{ARGS}})

cli *ARGS:
    #!/usr/bin/env sh
    (cd ./platune-cli && go run . {{ARGS}}) 