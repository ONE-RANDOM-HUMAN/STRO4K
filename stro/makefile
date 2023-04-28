EXE = stro
EVALFILE = ../nn-f28f93897b2f1437.nnue
build:
	RUSTFLAGS="-Ctarget-cpu=native" NN_PATH=$(EVALFILE) cargo rustc --release --bin stro --features=nn_path -- --emit link=$(EXE)
