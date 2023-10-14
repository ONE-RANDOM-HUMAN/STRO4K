EXE = stro
EVALFILE=../nn-e3a9863b410c4598.nnue
build:
	RUSTFLAGS="-Ctarget-cpu=native" NN_PATH=$(EVALFILE) cargo rustc --release --bin stro --features=nn_path_env -- --emit link=$(EXE)
