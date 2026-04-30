# Doc / parity gates (Unix). From repo root: `make doc-check`
.PHONY: doc-check
doc-check:
	./scripts/check-homepage-practice-fragment-sync.sh
	./scripts/check-chooser-phrasing.sh

.PHONY: sync-homepage-practice
sync-homepage-practice:
	./scripts/sync-homepage-practice-fragment.sh

.PHONY: blind-reader-check
blind-reader-check:
	python3 ./scripts/validate-blind-reader-sheet.py --allow-unscored
	# When the sheet has real ratings, run strict mode:
	# python3 ./scripts/validate-blind-reader-sheet.py

.PHONY: blind-reader-dry-run
blind-reader-dry-run:
	./scripts/blind-reader-dry-run.sh

.PHONY: play-contract-stress
play-contract-stress:
	python3 ./scripts/play-contract-stress.py

.PHONY: play-contract-smoke
play-contract-smoke:
	python3 ./scripts/play-contract-stress.py --smoke

.PHONY: worldcli-simulate-dialogue-smoke
worldcli-simulate-dialogue-smoke:
	./scripts/worldcli-simulate-dialogue-smoke.sh
