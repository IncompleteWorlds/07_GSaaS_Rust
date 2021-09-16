#!/bin/bash

#curl -i --request GET --url "http://localhost:11005/fdsaas/api/exit" --data '{"msg_code_id" : "exit", "authentication_key" : "00998844", "exit_code": "XYZZY" }'
curl -i --request GET --url "http://localhost:11005/fdsaas/api/exit" --data @test-modules/exit.json


