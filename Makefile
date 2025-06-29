SCRIPT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

otaclean:
	@echo "Cleaning old OTAs"
	find "$(SCRIPT_DIR)/../dist" \(  -name "*.zip" -or -name "*.zip.zst" \) -delete

distclean:
	@echo "Cleaning dist dir"
	rm -rf -- "$(SCRIPT_DIR)/../dist"
	mkdir -p -- "$(SCRIPT_DIR)/../dist"

.PHONY: clean distclean
