EXE = stro
EVALFILE = ../../nn-13f172599bc152d5.nnue
build:
	RUSTFLAGS="-Ctarget-cpu=native" NN_PATH=$(EVALFILE) cargo rustc --release --bin stro --features=nn_path -- --emit link=$(EXE)