setup:
	git fetch && git pull origin master
	git submodule update --init --recursive
	git submodule foreach git checkout origin/master
	git submodule foreach git checkout origin/develop

master:
	git fetch && git pull origin master
	git submodule update --init --recursive
	git submodule foreach git fetch
	git submodule foreach git checkout master
	git submodule foreach git pull origin master

develop:
	git fetch && git pull origin master
	git submodule update --init --recursive
	git submodule foreach git fetch
	git submodule foreach git checkout develop
	git submodule foreach git pull origin develop