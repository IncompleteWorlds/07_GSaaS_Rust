#!/usr/bin/bash

echo "Test create a new user"
nngcat --req --connect=tcp://127.0.0.1:13006  --file=test-modules/register_user_demo.json  --ascii


echo "Login"
nngcat --req --connect=tcp://127.0.0.1:13006  --file=test-modules/login.json  --ascii

echo "Logout"
nngcat --req --connect=tcp://127.0.0.1:13006  --file=test-modules/logout.json  --ascii

# nngcat --req --connect=tcp://127.0.0.1:13006  --file=test-modules/deregister_user_demo.json  --ascii