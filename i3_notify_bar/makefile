release: set-version test
	@echo "Compile in release mode"
	@cargo build --release
	@echo "Packing file"
	@mkdir -p release
	@rm -f release/i3_notify_bar.tar.gz
	@mv target/release/i3_notify_bar release/i3_notify_bar
	@strip "release/i3_notify_bar"
	@tar czf release/i3_notify_bar.tar.gz release/i3_notify_bar
	@echo
	@echo "Binary size: $$(wc -c release/i3_notify_bar)"
	@echo "SHA1: $$(sha1sum release/i3_notify_bar.tar.gz)" | tee --append release/checksums.txt
	@echo "SHA256: $$(sha256sum release/i3_notify_bar.tar.gz)" | tee --append release/checksums.txt
	@echo "md5: $$(md5sum release/i3_notify_bar.tar.gz)" | tee --append release/checksums.txt


set-version:
	@echo -n "current version:"
	@cat version.txt;
	@echo ""
	@read -p "New version: " VERSION; \
	git tag -a V$$VERSION -m "Version $$VERSION"; \
	echo -n $$VERSION > version.txt; \
	sed -i "s/^version = ".*"$$/version = \"$$VERSION\"/g" Cargo.toml

test:
	@echo "Running tests"
	@cargo test