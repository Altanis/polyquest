nodemon --watch server --watch shared --ext rs --exec "\
cd server && \
cargo run --release"