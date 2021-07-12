install:
	cargo build --release
	mv -v -- target/release/interview /usr/local/bin/
	cp -v -- etc/bash_completion.d/interview /usr/local/etc/bash_completion.d/interview
