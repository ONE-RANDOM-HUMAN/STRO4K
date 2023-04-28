EXE = stro
EVALFILE = ../nn-3249e62b0b08353f.nnue
build:
	RUSTFLAGS="-Ctarget-cpu=native" NN_PATH=$(EVALFILE) cargo rustc --release --bin stro --features=nn_path -- --emit link=$(EXE)
