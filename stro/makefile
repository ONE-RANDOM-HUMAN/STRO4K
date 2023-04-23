EXE = stro
EVALFILE = ../nn-72bee0bb01c8694b.nnue
build:
	RUSTFLAGS="-Ctarget-cpu=native" NN_PATH=$(EVALFILE) cargo rustc --release --bin stro --features=nn_path -- --emit link=$(EXE)