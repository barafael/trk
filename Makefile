sync-hook:
	rm ./.git/hooks/post-commit
	cp ./post-commit ./.git/hooks/post-commit

