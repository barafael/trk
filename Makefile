sync-hook:
	rm ./.git/hooks/post-commit
	cp ./post-commit ./.git/hooks/post-commit
	
	rm ./.git/hooks/post-checkout
	cp ./post-checkout ./.git/hooks/post-checkout
