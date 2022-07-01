with (import <nixpkgs> {});
let
  my-python = pkgs.python37;
  python-with-my-packages = my-python.withPackages (p: with p; [
	ecdsa
        fastecdsa
        sympy
	pip
	setuptools
  ]);

  basePackages = [
	pkgs.bashInteractive
	git
	direnv
	cargo
	rustc

	python-with-my-packages
	# required by fastecdsa
	gmp
  ];

  inputs = basePackages
    ++ lib.optional stdenv.isLinux inotify-tools
    ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        CoreFoundation
        CoreServices
      ]);

in mkShell {
  buildInputs = inputs;
  shellHook =
  ''
  export PIP_PREFIX="$(pwd)/_build/pip_packages"
  export PYTHONPATH="$(pwd)/_build/pip_packages/lib/python3.7/site-packages:$PYTHONPATH" 
  unset SOURCE_DATE_EPOCH
  pip3.7 install cairo-lang
  '';
}
